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

fn md5_password(username: &String, password: &String, salt: Vec<u8>) -> String {
    let mut md5 = Md5::new();
    md5.update(password);
    md5.update(username);
    let result = md5.finalize_reset();
    md5.update(format!("{:x}", result));
    md5.update(salt);
    format!("md5{:x}", md5.finalize())
}

pub struct ProtocolHandler {
    policy_watcher: watch::Receiver<Vec<u8>>,
    client_conn: PostgresConn,
    target_conn: PostgresConn,
    policy_evaluator: Evaluator,
    groups: Vec<String>
}

impl ProtocolHandler {
    // serve will listen to client packets and decide whether to process
    // the packet based on the opa policy.
    pub async fn serve(&mut self) {
        let mut target_buf = [0;1024];
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
                n = self.target_conn.read(&mut target_buf) => {
                    match n {
                        Err(e) =>{
                                println!("failed to read from socket; err = {:?}", e);
                                return ();
                        },
                        Ok(n) =>{
                            if n == 0 {
                                return ()
                            }
                            if let Err(e) = self.client_conn.write_all(&target_buf[0..n]).await{
                                return
                            }
                        }
                    }
                }
                n = FrontendMessage::decode(&mut self.client_conn) => {
                    match n {
                        Err(e) =>{
                                println!("failed to read from socket; err = {:?}", e);
                                return ();
                        },
                        Ok(msg) =>{
                            debug!("got frontend message {:?}", msg);
                            if let Err(e) = self.target_conn.write_all(&msg.encode()).await{
                                return;
                            }
                        }
                    }
                }
            }
        }
    }

    // intialize will create a new connection with target and returns initialized postgres protocol handler.
    pub async fn initialize(
        config: PostgresConfig,
        mut client_conn: PostgresConn,
        client_parms: HashMap<String, String>,
        policy_watcher: watch::Receiver<Vec<u8>>,
        groups: Vec<String>
    ) -> Result<ProtocolHandler, anyhow::Error> {
        debug!("intializing protocol handler");
        let mut target_conn = ProtocolHandler::connect_target(&config).await?;
        target_conn = ProtocolHandler::try_ssl_upgrade(&config, target_conn).await?;

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
                &FrontendMessage::Startup {
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
                        .write_all(&FrontendMessage::PasswordMessage { password }.encode())
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
                            &FrontendMessage::PasswordMessage {
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
                BackendMessage::AuthenticationOk { .. } => {
                    // send authentication ok to client connection since we established connection with
                    // target.
                    client_conn.write_all(&rsp_msg.encode()).await?;
                    let cp = policy_watcher.clone();
                    let wasm_policy = cp.borrow();
                    let evaluator = Evaluator::new(
                        String::from("inspecktor-policy"),
                        &wasm_policy,
                        &DEFAULT_HOST_CALLBACKS,
                    )
                    .unwrap();
                    let handler = ProtocolHandler {
                        target_conn: target_conn,
                        client_conn: client_conn,
                        policy_watcher: policy_watcher,
                        policy_evaluator: evaluator,
                        groups: groups,
                    };
                    return Ok(handler);
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
                    .write_all(&FrontendMessage::SslRequest.encode())
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
                    return ProtocolHandler::connect_target(config).await;
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
