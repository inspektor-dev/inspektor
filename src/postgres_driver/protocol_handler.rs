use std::net::SocketAddr;

use crate::config::PostgresConfig;
use crate::postgres_driver::conn::PostgresConn;
use crate::postgres_driver::message::*;
use anyhow::*;
use log::*;
use md5::{Digest, Md5};
use openssl::ssl::{SslConnector, SslMethod};
use std::collections::HashMap;
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_openssl::SslStream;
pub struct ProtocolHandler {
    pub config: PostgresConfig,
    pub remote_conn: Option<PostgresConn>,
    pub client_conn: PostgresConn,
}

impl ProtocolHandler {
    // init should be called after authenticated client connection. init will try to connect with remote
    // server. if not possible then it'll close the client connection.
    pub async fn init(
        mut self,
        client_params: HashMap<String, String>,
    ) -> Result<(), anyhow::Error> {
        debug!(
            "initializing postgres protocol handler with config {:?}",
            self.config
        );
        let mut target_conn = TcpStream::connect(self.config.target_addr.as_ref().unwrap())
            .await
            .map_err(|e| {
                error!("error while reaching to target server {:?}", e);
                return anyhow!("unable to connect to remote server");
            })?;

        let mut target_conn = PostgresConn::Unsecured(target_conn);

        target_conn
            .write_all(&StartupMessage::SslRequest.encode())
            .await
            .map_err(|e| {
                error!("error while writing ssl request to remote target {:?}", e);
                return anyhow!("unable to connect to remote serveer");
            })?;

        // check whether target server support ssl.
        let mut buf = [0; 1];
        target_conn.read_exact(&mut buf).await.map_err(|e| {
            error!("error while reading the ssl response message {:?}", e);
            return anyhow!("unable to reach remote server");
        })?;

        // check whether target accepts ssl connection.
        if buf[0] == ACCEPT_SSL_ENCRYPTION {
            error!("upgrading target connection to ssl connection");
            // let's upgrade the connection into secure one.
            match target_conn {
                PostgresConn::Unsecured(inner) => {
                    let mut connector = SslConnector::builder(SslMethod::tls())
                        .unwrap()
                        .build()
                        .configure()
                        .unwrap()
                        .verify_hostname(false)
                        .use_server_name_indication(false)
                        .into_ssl("")
                        .unwrap();

                    let mut stream = SslStream::new(connector, inner).unwrap();
                    Pin::new(&mut stream).connect().await.map_err(|e| {
                        error!(
                            "unable to upgrade the target connection to ssl stream {:?}",
                            e
                        );
                        anyhow!("unbale to reach remote server")
                    })?;
                    target_conn = PostgresConn::Secured(stream)
                }
                _ => {
                    unreachable!("can't upgrade the connection which is already secured")
                }
            }
        } else {
            // target doesn't looks like accpeting tls connectio. so let's try to connect with the target again.
            let new_conn = TcpStream::connect(self.config.target_addr.as_ref().unwrap())
                .await
                .map_err(|e| {
                    error!("error while reaching to target server {:?}", e);
                    return anyhow!("unable to connect to remote server");
                })?;
            target_conn = PostgresConn::Unsecured(new_conn);
        }
        // Initiate the startup message.
        let mut target_params = HashMap::new();
        target_params.insert(
            "database".to_string(),
            client_params.get("database").unwrap().clone(),
        );
        target_params.insert(
            "user".to_string(),
            self.config.target_username.as_ref().unwrap().clone(),
        );
        target_params.insert("client_encoding".to_string(), "UTF8".to_string());
        target_params.insert("application_name".to_string(), "inspektor".to_string());

        let msg = StartupMessage::Startup {
            params: target_params,
            version: VERSION_3,
        };
        target_conn.write_all(&msg.encode()).await.map_err(|e| {
            error!("error while sending startup message to the target {:?}", e);
            anyhow!("error while connecting to target database")
        })?;

        // handle the authentication method.
        let msg = decode_backend_message(&mut target_conn)
            .await
            .map_err(|err| {
                error!(
                    "error while decoding the first message after startup message {:?}",
                    err
                );
                return anyhow!("invalid target message");
            })?;

        debug!("got target message {:?} after startup message", msg);
        match msg {
            BackendMessage::StartupMessage(inner) => match inner {
                StartupMessage::AuthenticationMD5Password { salt } => {
                    let password = md5_password(
                        self.config.target_username.as_ref().unwrap(),
                        self.config.target_password.as_ref().unwrap(),
                        salt,
                    );
                    let password_msg = FrotendMessage::PasswordMessage { password };
                    target_conn
                        .write_all(&password_msg.encode())
                        .await
                        .map_err(|e| {
                            error!("error while sending password message: [err msg: {:?}]", e);
                            anyhow!("error while sending password messaage to target")
                        })?;
                }
                StartupMessage::AuthenticationCleartextPassword => {
                    let password = self.config.target_username.as_ref().unwrap().clone();
                    let password_msg = FrotendMessage::PasswordMessage { password };
                    target_conn
                        .write_all(&password_msg.encode())
                        .await
                        .map_err(|e| {
                            error!("error while sending password message: [err msg: {:?}]", e);
                            anyhow!("error while sending password messaage to target")
                        })?;
                }
                _ => {
                    error!("expected password authentication message ");
                    return Err(anyhow!("invalid target message"));
                }
            },
            _ => {
                error!("expected password authentication message ");
                return Err(anyhow!("invalid target message"));
            }
        }

        // we have done the authentication let's wait for authentication ok message.
        let msg = decode_backend_message(&mut target_conn)
            .await
            .map_err(|err| {
                error!(
                    "error while decoding the first message after startup message {:?}",
                    err
                );
                return anyhow!("invalid target message");
            })?;
        match msg {
            BackendMessage::StartupMessage(inner) => match inner {
                StartupMessage::AuthenticationOk { success } => {
                    if !success {
                        error!("authentication failed with the target server");
                        return Err(anyhow!("unable to reach target server"));
                    }
                }
                _ => {
                    error!(
                        "expected authentication ok message from target but got {:?}",
                        inner
                    );
                    return Err(anyhow!("invalid target message"));
                }
            },
            _ => {
                error!(
                    "expected authentication ok message from target but got {:?}",
                    msg
                );
                return Err(anyhow!("invalid target message"));
            }
        }

        let mut target_buf = [0; 1024];
        let mut client_buf = [0; 1024];
        loop {
            tokio::select! {
                n = target_conn.read(&mut target_buf) =>{
                    match n {
                        Err(e) =>{
                                println!("failed to read from socket; err = {:?}", e);
                                return Ok(());
                        },
                        Ok(n) =>{
                            if n == 0 {
                                return Ok(())
                            }
                            self.client_conn.write_all(&target_buf[0..n]).await?
                        }
                    }
                }
                n = self.client_conn.read(&mut client_buf) => {
                    match n {
                        Err(e) =>{
                                println!("failed to read from socket; err = {:?}", e);
                                return Ok(());
                        },
                        Ok(n) =>{
                            if n == 0 {
                                return Ok(())
                            }
                            target_conn.write_all(&client_buf[0..n]).await?
                        }
                    }
                }
            }
        }


        // // let's pipe both the connection and see what happens :P
        // let mut buf = [0; 1024];
        // // In a loop, read data from the socket and write the data back.
        // loop {
        //     let n = match target_conn.read(&mut buf).await {
        //         // socket closed
        //         Ok(n) if n == 0 => return Ok(()),
        //         Ok(n) => n,
        //         Err(e) => {
        //             eprintln!("failed to read from socket; err = {:?}", e);
        //             return Ok(());
        //         }
        //     };

        //     // Write the data back
        //     if let Err(e) = self.client_conn.write_all(&buf[0..n]).await {
        //         eprintln!("failed to write to socket; err = {:?}", e);
        //         return Ok(());
        //     }
        // }
        Ok(())
    }
}

fn md5_password(username: &String, password: &String, salt: Vec<u8>) -> String {
    let mut md5 = Md5::new();
    md5.update(password);
    md5.update(username);
    let result = md5.finalize_reset();
    md5.update(format!("{:x}", result));
    md5.update(salt);
    format!("md5{:x}", md5.finalize())
}
