use crate::postgres_driver::codec::*;
use crate::postgres_driver::errors::DecoderError;
use crate::postgres_driver::message::StartupMessage;
use anyhow::anyhow;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use log::*;
use tokio;
use tokio::net::TcpListener;
pub struct PostgresDriver {}

impl PostgresDriver {
    pub fn start(&self) {
        // run the socket message.
        let mut rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let listener = TcpListener::bind(&"127.0.0.1:8080".to_string())
                .await
                .map_err(|_| anyhow!("unable to listern on the given port"))
                .unwrap();
            info!("postgres driver listeneing at 127.0.0.1:8080");
            loop {
                let (mut socket, _) = listener.accept().await.unwrap();
                tokio::spawn(async move {
                    let msg = match decode_startup_message(&mut socket).await {
                        Ok(msg) => msg,
                        Err(e) => {
                            match e {
                                DecoderError::UnsupporedVersion => {
                                    error!("closing connection because of unsuppored version");
                                    // looks like client sent lower version.
                                    // report that and close the connection.
                                    return;
                                }
                                _ => {
                                    error!("error while decoding startup message {:?}", e);
                                    // log the error and close the connection.
                                    return;
                                }
                            };
                        }
                    };
                    debug!("we got startup message{:?}", msg);
                    match msg {
                        StartupMessage::Startup { params, .. } => {
                            // we have to ask for passcode after connecting.
                            let buf = StartupMessage::AuthenticationCleartextPassword.encode();
                            if let Err(e) = socket.write(&buf).await{
                                error!("error while sending AuthenticationCleartextPassword {:?}", e);
                                return;
                            }
                            let result = decode_password_message(&mut socket).await;
                            if result.is_err(){
                                error!("error while decoding password message {:?}", result.unwrap_err());
                                return;
                            }
                            if let StartupMessage::PasswordMessage {password} = result.unwrap(){
                                // send authetication ok message and handle the query request from here.
                                if let Err(e) = socket.write(&StartupMessage::AuthenticationOk.encode()).await{
                                    error!("erropr while writing authentication ok message {:?}", e);
                                    return;
                                }
                                return;
                            }
                            unreachable!("message expected to be password message");
                        }
                        _ => {
                            error!("dropping connection because of unrecognized msg {:?}", msg);
                            return;
                        }
                    }
                });
            }
        });
    }
}
