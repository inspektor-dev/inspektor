use std::fs::File;
use std::io::Read;
use std::ops::DerefMut;
use std::pin::Pin;

use crate::postgres_driver::codec::*;
use crate::postgres_driver::errors::DecoderError;
use crate::postgres_driver::message::StartupMessage;
use crate::postgres_driver::conn::PostgresConn;
use anyhow::anyhow;
use log::*;
use std::sync::Arc;
use tokio;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, Interest};
use tokio::net::TcpListener;
use tokio_native_tls::native_tls::{Identity, TlsAcceptor};
use tokio_native_tls::TlsAcceptor as TokioTlsAcceptor;

pub struct PostgresDriver {}

impl PostgresDriver {
    pub fn start(&self) {
        let mut file = File::open("/home/poonai/inspektor/identity.p12").unwrap();
        let mut identity = vec![];
        file.read_to_end(&mut identity).unwrap();
        let identity = Identity::from_pkcs12(&identity, "mypass").unwrap();
        // run the socket message.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let listener = TcpListener::bind(&"127.0.0.1:8080".to_string())
                .await
                .map_err(|_| anyhow!("unable to listern on the given port"))
                .unwrap();
            info!("postgres driver listeneing at 127.0.0.1:8080");
            let acceptor = TlsAcceptor::new(identity).unwrap();
            let acceptor = Arc::new(TokioTlsAcceptor::from(acceptor));
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let acceptor = acceptor.clone();
                let mut socket = PostgresConn::Unsecured(socket);
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
                            let mut socket = acceptor.accept(socket).await.unwrap();
                            // we have to ask for passcode after connecting.
                            let buf = StartupMessage::AuthenticationCleartextPassword.encode();
                            if let Err(e) = socket.write(&buf).await {
                                error!(
                                    "error while sending AuthenticationCleartextPassword {:?}",
                                    e
                                );
                                return;
                            }
                            let result = decode_password_message(&mut socket).await;
                            if result.is_err() {
                                error!(
                                    "error while decoding password message {:?}",
                                    result.unwrap_err()
                                );
                                return;
                            }
                            if let StartupMessage::PasswordMessage { password } = result.unwrap() {
                                // send authetication ok message and handle the query request from here.
                                if let Err(e) = socket
                                    .write(&StartupMessage::AuthenticationOk.encode())
                                    .await
                                {
                                    error!(
                                        "erropr while writing authentication ok message {:?}",
                                        e
                                    );
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



