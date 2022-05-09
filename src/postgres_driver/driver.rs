use std::collections::HashMap;

use crate::apiproto::api::{AuthRequest, DataSourceResponse, AuthResponse};
use crate::apiproto::api_grpc::*;
use crate::config::PostgresConfig;
use crate::policy_evaluator::evaluator::PolicyEvaluator;
use crate::postgres_driver::conn::PostgresConn;
use crate::postgres_driver::errors::DecoderError;
use crate::postgres_driver::message::*;
use crate::postgres_driver::protocol_handler::*;
use crate::postgres_driver::utils::*;
use anyhow::anyhow;
use grpcio::CallOption;
use log::*;
use openssl::ssl::{Ssl, SslAcceptor, SslFiletype, SslMethod};

use std::pin::Pin;

use tokio;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::watch;
use tokio_openssl::SslStream;
use tokio::sync::mpsc::Sender;
#[derive(Clone)]
pub struct PostgresDriver {
    pub postgres_config: PostgresConfig,
    pub policy_watcher: watch::Receiver<Vec<u8>>,
    pub client: InspektorClient,
    pub token: String,
    pub datasource: DataSourceResponse,
    pub audit_sender: Sender<String>
}

impl PostgresDriver {
    pub fn start(&self) {
        // let acceptor = self.get_ssl_acceptor();
        // run the socket message.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let listener = TcpListener::bind(format!(
                "0.0.0.0:{}",
                self.postgres_config.proxy_listen_port.as_ref().unwrap()
            ))
            .await
            .map_err(|_| anyhow!("unable to listern on the given port"))
            .unwrap();        
            info!(
                "postgres driver listeneing at 0.0.0.0:{}",
                self.postgres_config.proxy_listen_port.as_ref().unwrap()
            );
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                //       let acceptor = acceptor.clone();
                let driver = self.clone();
                let socket = PostgresConn::Unsecured(socket);
                tokio::spawn(async move {
                    driver.handle_client_conn(socket, None).await;
                    ()
                });
            }
        });
    }

    // get_ssl_acceptor will get ssl acceptor if the sidecar is set to run on tls
    // mode.
    // fn get_ssl_acceptor(&self) -> SslAcceptor {
    //     let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    //     acceptor
    //         .set_private_key_file("/home/poonai/inspektor/key.pem", SslFiletype::PEM)
    //         .unwrap();
    //     acceptor
    //         .set_certificate_chain_file("/home/poonai/inspektor/cert.pem")
    //         .unwrap();
    //     let acceptor = acceptor.build();
    //     acceptor
    // }

    async fn handle_client_conn(
        &self,
        mut client_conn: PostgresConn,
        acceptor: Option<SslAcceptor>,
    ) {
        loop {
            // get the initial startup message from client.
            let msg = match decode_init_startup_message(&mut client_conn).await {
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

            match msg {
                FrontendMessage::Startup { params, .. } => {
                    // let's verify the user name and
                    let auth_res =
                        match self.verfiy_client_params(&params, &mut client_conn).await {
                            Ok(result) => result,
                            Err(e) => {
                                error!("error while verifying auth. err msg: {:?}", e);
                                continue;
                            }
                        };
                    // check whether user can access the db.
                    let mut evaluator = match PolicyEvaluator::new(&self.policy_watcher.borrow()) {
                        Ok(evaluator) => evaluator,
                        Err(e) => {
                            error!("error while building the policy evaluator {:?}", e);
                            return;
                        }
                    };

                    let groups: Vec<String> = auth_res.get_groups().into();
                    let result = match evaluator.evaluate(
                        &self.datasource.data_source_name,
                        &"view".to_string(),
                        &groups,
                    ) {
                        Ok(res) => res,
                        Err(e) => {
                            error!("error while evulating policy {:?}", e);
                            return;
                        }
                    };
                    if !result.allow {
                        // since this datasource is not allowed by the group
                        // let's drop the connection here.
                        info!("incomming connection don't have access to the given datasource");
                        return;
                    }

                    // terminate the connection if the incoming db access is fall under protected
                    // attribute.
                    if let Some(_) = result
                        .protected_attributes
                        .iter()
                        .position(|attribute| attribute == params.get("database").unwrap())
                    {
                        error!("unautorized db access");
                        return;
                    }

                    let mut handler = match ProtocolHandler::initialize(
                        self.postgres_config.clone(),
                        client_conn,
                        params,
                        self.policy_watcher.clone(),
                        groups,
                        evaluator,
                        self.datasource.data_source_name.clone(),
                        self.client.clone(),
                        self.token.clone(),
                        auth_res.passthrough
                    )
                    .await
                    {
                        Ok(h) => h,
                        Err(e) => {
                            error!("error while initializing protocol handler {:?}", e);
                            return;
                        }
                    };
                    handler.serve(auth_res.expires_at).await.unwrap();
                    return;
                }
                FrontendMessage::SslRequest => {
                    if let None = acceptor {
                        debug!("dropping connection since we are not able to upgrade the connection to ssl");
                        return;
                    }
                    //
                    if let PostgresConn::Unsecured(mut inner) = client_conn {
                        // tell the client that you are upgrading for secure connection
                        if let Err(e) = inner.write_all(&[ACCEPT_SSL_ENCRYPTION]).await {
                            error!(
                                "error while sending ACCEPT_SSL_ENCRYPTION to client {:?}",
                                e
                            );
                            return;
                        }
                        let ssl = Ssl::new(acceptor.as_ref().unwrap().context()).unwrap();
                        let mut stream = SslStream::new(ssl, inner).unwrap();
                        Pin::new(&mut stream).accept().await.unwrap();
                        client_conn = PostgresConn::Secured(stream);
                        debug!("client connection upgraded  to tls connection");
                        continue;
                    }
                    // upgrade the client connection to secured tls connection.
                    error!("can't upgrade secured connection. sus client");
                    return;
                }
                _ => {
                    error!("don't know the message type");
                    // all the return should send a error message before closing.
                    return;
                }
            }
        }
    }

    // verfiy_client_params will verify the client password with the dataplane.
    // if it's succeed it'll retrive all group assigned to the user.
    async fn verfiy_client_params(
        &self,
        params: &HashMap<String, String>,
        client_conn: &mut PostgresConn,
    ) -> Result<AuthResponse, anyhow::Error> {
        let buf = BackendMessage::AuthenticationCleartextPassword.encode();
        client_conn.write_all(&buf).await.map_err(|e| {
            error!(
                "error while writing clear text password message to the client. err: {:?}",
                e
            );
            e
        })?;

        let msg = decode_password_message(client_conn).await?;

        let password = match msg {
            FrontendMessage::PasswordMessage { password } => password,
            _ => {
                unreachable!("expectected password message while decoding for password message");
            }
        };
        let mut auth_req = AuthRequest::new();
        auth_req.password = password;
        auth_req.user_name = params.get("user").unwrap().clone();
        let res = self.client.auth_opt(&auth_req, self.get_call_opt())?;
        Ok(res)
    }

    fn get_call_opt(&self) -> CallOption {
        let mut meta_builder = grpcio::MetadataBuilder::new();
        meta_builder
            .add_str("auth-token", self.token.as_ref())
            .unwrap();
        let meta = meta_builder.build();
        return grpcio::CallOption::default().headers(meta);
    }
}
