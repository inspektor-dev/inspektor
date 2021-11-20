use std::fs::File;
use std::io::Read;


use crate::postgres_driver::utils::*;
use crate::postgres_driver::conn::PostgresConn;
use crate::postgres_driver::errors::DecoderError;
use crate::postgres_driver::message::*;
use crate::config::PostgresConfig;
use anyhow::anyhow;
use log::*;
use std::path::Path;
use std::sync::Arc;
use tokio;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, Interest};
use tokio::net::TcpListener;
use std::io::{self, BufReader};
use openssl::ssl::{Ssl, SslAcceptor, SslConnector, SslFiletype, SslMethod};
use tokio_openssl::SslStream;
use std::pin::Pin;
use std::time::Duration;
use crate::postgres_driver::protocol_handler;

pub struct PostgresDriver {
    pub postgres_config: PostgresConfig
}

impl PostgresDriver {
    pub fn start(&self) {
        let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        acceptor
            .set_private_key_file("/home/poonai/inspektor/key.pem", SslFiletype::PEM)
            .unwrap();
        acceptor
            .set_certificate_chain_file("/home/poonai/inspektor/cert.pem")
            .unwrap();
        let acceptor = acceptor.build();
      

        // run the socket message.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let listener = TcpListener::bind(&"127.0.0.1:8080".to_string())
                .await
                .map_err(|_| anyhow!("unable to listern on the given port"))
                .unwrap();
            info!("postgres driver listeneing at 127.0.0.1:8080");
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                let acceptor = acceptor.clone();
                let mut socket = PostgresConn::Unsecured(socket);
                tokio::spawn(async move {
                    loop {
                        let msg = match decode_init_startup_message(&mut socket).await {
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
                            FrotendMessage::Startup { params, .. } => {
                                // we have to ask for passcode after connecting.
                                let buf = BackendMessage::AuthenticationCleartextPassword.encode();
                                if let Err(e) = socket.write_all(&buf).await {
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
                                };
                                if let FrotendMessage::PasswordMessage{ password } =
                                    result.unwrap()
                                {
                                    // send authetication ok message and handle the query request from here.
                                    if let Err(e) = socket
                                        .write(&BackendMessage::AuthenticationOk{success: true}.encode())
                                        .await
                                    {
                                        error!(
                                            "erropr while writing authentication ok message {:?}",
                                            e
                                        );
                                        return;
                                    }
                                    println!("aquired password {:?}", password);
                                    let mut handler = protocol_handler::ProtocolHandler{
                                        config: PostgresConfig::default(),
                                        remote_conn: None,
                                        client_conn: socket
                                    };
                                    handler.init(params).await.unwrap();
                                    return;
                                }
                                unreachable!("message expected to be password message");
                            },
                            FrotendMessage::SslRequest =>{
                                if let PostgresConn::Unsecured(mut inner) = socket{
                                    // tell the client that you are upgrading for secure connection
                                    if let Err(e) = inner.write_all(&[ACCEPT_SSL_ENCRYPTION]).await{
                                        error!("error while sending ACCEPT_SSL_ENCRYPTION to client {:?}", e);
                                        return;
                                    }
                                    let ssl = Ssl::new(acceptor.context()).unwrap();
                                    let mut stream = SslStream::new(ssl, inner).unwrap();
                                    Pin::new(&mut stream).accept().await.unwrap();
                                    socket = PostgresConn::Secured(stream);
                                    debug!("client connection upgraded  to tls connection");
                                    continue;
                                }
                                error!(
                                    "connection can't be secured when client ask for tls connection",
                                );
                                return;
                            }
                            _ => {
                                error!("dropping connection because of unrecognized msg {:?}", msg);
                                return;
                            }
                        }
                    }
                });
            }
        });
    }
}
