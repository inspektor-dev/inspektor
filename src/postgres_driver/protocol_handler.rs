use std::mem;
use std::net::{SocketAddr, TcpListener};

use crate::config::PostgresConfig;
use crate::postgres_driver::conn::PostgresConn;
use crate::postgres_driver::message::*;
use anyhow::*;
use burrego::opa::host_callbacks::DEFAULT_HOST_CALLBACKS;
use burrego::opa::wasm::Evaluator;
use log::*;
use md5::{Digest, Md5};
use openssl::ssl::{SslConnector, SslMethod};
use std::collections::HashMap;
use std::pin::Pin;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::watch;
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
            .write_all(&FrotendMessage::SslRequest.encode())
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

        let msg = FrotendMessage::Startup {
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
            BackendMessage::AuthenticationMD5Password { salt } => {
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
            BackendMessage::AuthenticationCleartextPassword => {
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
            BackendMessage::AuthenticationOk { success } => {
                if !success {
                    error!("authentication failed with the target server");
                    return Err(anyhow!("unable to reach target server"));
                }
            }
            _ => {
                error!(
                    "expected authentication ok message from target but got {:?}",
                    msg
                );
                return Err(anyhow!("invalid target message"));
            }
        }

        let mut target_buf = [0; 1024];
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
                n = FrotendMessage::decode(&mut self.client_conn) => {
                    match n {
                        Err(e) =>{
                                println!("failed to read from socket; err = {:?}", e);
                                return Ok(());
                        },
                        Ok(msg) =>{
                            debug!("got frontend message {:?}", msg);
                            target_conn.write_all(&msg.encode()).await?;
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


struct ProtocolHandlerNew {
    policy_evaluator: Evaluator,
    policy_watcher: watch::Receiver<Vec<u8>>,
    client_conn: PostgresConn,
    target_conn: PostgresConn,
}

impl ProtocolHandlerNew {
    // serve will listen to client packets and decide whether to process
    // the packet based on the opa policy.
    async fn serve(&mut self) {
        loop {
            tokio::select! {
                evaluator = self.policy_watcher.changed() => {
                    if !evaluator.is_ok(){
                        error!("watched failed to get new evaluation. prolly watcher closed");
                        continue;
                    }
                    let wasm_policy = self.policy_watcher.borrow();
                    // update the current evaluator with new policy
                    let evaluator = Evaluator::new(String::from("inspecktor-policy"), &wasm_policy, &DEFAULT_HOST_CALLBACKS).unwrap();
                    self.policy_evaluator = evaluator;
                }
            }
        }
    }

    // intialize will create a new connection with target and returns initialized postgres protocol handler.
    async fn initialize(
        config: PostgresConfig,
        client_conn: PostgresConn,
        client_parms: HashMap<String, String>,
        policy_evaluator: Evaluator,
        policy_watcher: watch::Receiver<Vec<u8>>
    ) -> Result<ProtocolHandlerNew, anyhow::Error> {
        debug!("intializing protocol handler");
        let mut target_conn = ProtocolHandlerNew::connect_target(&config).await?;
        target_conn = ProtocolHandlerNew::try_ssl_upgrade(&config, target_conn).await?;

        // create startup parameter to establish authenticated connection.
        let startup_params = HashMap::from([
            (
                "database".to_string(),
                client_parms.get("database").unwrap().clone(),
            ),
            (
                "user".to_string(),
                config.target_username.as_ref().unwrap().clone(),
            ),
            ("client_encoding".to_string(), "UTF8".to_string()),
            ("application_name".to_string(), "inspektor".to_string()),
        ]);
        target_conn
            .write_all(
                &FrotendMessage::Startup {
                    params: startup_params,
                    version: VERSION_3,
                }
                .encode(),
            )
            .await
            .map_err(|e| {
                error!(
                    "error while sending startup message to target. err: {:?}",
                    e
                );
                e
            })?;

        // send password if the target ask's for otherwise wait for the
        // AuthenticationOk message;
        loop {
            let rsp_msg = decode_backend_message(&mut target_conn)
                .await
                .map_err(|e| {
                    error!("error decoding target message. error {:?}", e);
                    e
                })?;
            match rsp_msg {
                BackendMessage::AuthenticationMD5Password { salt } => {
                    let password = md5_password(
                        config.target_username.as_ref().unwrap(),
                        config.target_password.as_ref().unwrap(),
                        salt,
                    );
                    target_conn
                        .write_all(&FrotendMessage::PasswordMessage { password }.encode())
                        .await
                        .map_err(|e| {
                            error!("error while sending md5 password message to target");
                            e
                        })?;
                    continue;
                }
                BackendMessage::AuthenticationCleartextPassword => {
                    target_conn
                        .write_all(
                            &FrotendMessage::PasswordMessage {
                                password: config.target_password.as_ref().unwrap().clone(),
                            }
                            .encode(),
                        )
                        .await
                        .map_err(|e| {
                            error!("error while sending password message to target");
                            e
                        })?;
                    continue;
                }
                BackendMessage::AuthenticationOk{..} => {
                    let handler = ProtocolHandlerNew{
                        target_conn: target_conn,
                        client_conn: client_conn,
                        policy_evaluator: policy_evaluator,
                        policy_watcher: policy_watcher,
                    };
                    return Ok(handler)
                }
                _ => {
                    error!(
                        "got unexpected backend message from backend. msg{:?}",
                        rsp_msg
                    );
                    return Err(anyhow!("unexpected backend message from target"));
                }
            }
        }
    }

    // connect_target will create an unsecured connection with target postgres instance.
    async fn connect_target(config: &PostgresConfig) -> Result<PostgresConn, anyhow::Error> {
        Ok(PostgresConn::Unsecured(
            TcpStream::connect(config.target_addr.as_ref().unwrap())
                .await
                .map_err(|e| {
                    error!(
                        "error while creating tcp connection with target postgres. err: {:?}",
                        e
                    );
                    return anyhow!("unable to connect to target postgres server");
                })?,
        ))
    }

    // try_ssl_upgrade will try to upgrade the unsecured postgres connection to ssl connection
    // if the server supports. Otherwise, unsercured connection is retured back.
    async fn try_ssl_upgrade(
        config: &PostgresConfig,
        conn: PostgresConn,
    ) -> Result<PostgresConn, anyhow::Error> {
        match conn {
            PostgresConn::Unsecured(mut inner) => {
                inner
                    .write_all(&FrotendMessage::SslRequest.encode())
                    .await
                    .map_err(|e| {
                        error!("unable to send ssl upgrade request to target. err: {:?}", e);
                        return anyhow!("unable to send ssl upgrade request");
                    })?;
                // check whether remote server accept ssl connection.
                let mut buf = [0; 1];
                inner.read_exact(&mut buf).await.map_err(|e| {
                    error!("error reading response message after ssl request {:?}", e);
                    return anyhow!("error while reading response message after ssl request");
                })?;
                if buf[0] != ACCEPT_SSL_ENCRYPTION {
                    // since postgres doesn't accept ssl. so let's drop the
                    // current connection and create a new unsecured connection.
                    return ProtocolHandlerNew::connect_target(config).await;
                }
                let connector = SslConnector::builder(SslMethod::tls())
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
                    anyhow!("error while upgrading target connection to ssl stream")
                })?;
                Ok(PostgresConn::Secured(stream))
            }
            _ => Ok(conn),
        }
    }
}
