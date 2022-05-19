use std::collections::HashMap;

use crate::apiproto::api::{AuthRequest, AuthResponse, DataSourceResponse};
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
use tokio::sync::mpsc::Sender;
use tokio::sync::watch;
use tokio_openssl::SslStream;
#[derive(Clone)]
pub struct PostgresDriver {
    pub postgres_config: PostgresConfig,
    pub policy_watcher: watch::Receiver<Vec<u8>>,
    pub client: InspektorClient,
    pub token: String,
    pub datasource: DataSourceResponse,
    pub audit_sender: Sender<String>,
    pub ssl_acceptor: Option<SslAcceptor>,
}

impl PostgresDriver {

    /// start will start listening for postgres connection from the configured
    /// listening port.
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
                    if let Err(e) = driver.handle_client_conn(socket).await {
                        error!("error while handling client connection {:?}", e);
                    }
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

    /// handle_client_conn will handle the tcp connection of the client.
    async fn handle_client_conn(&self, conn: PostgresConn) -> Result<(), anyhow::Error> {
        let (startup_msg, mut conn) = self.get_startup_msg(conn).await?;

        let params = match startup_msg {
            FrontendMessage::Startup { params, .. } => params,
            _ => unreachable!(),
        };
        // authenticate client connection.
        let auth_res = match self.verfiy_client_params(&params, &mut conn).await {
            Ok(result) => result,
            Err(e) => return Err(anyhow!("error while verifying auth. err msg: {:?}", e)),
        };

        // check whether user can access the db.
        let mut evaluator = match PolicyEvaluator::new(&self.policy_watcher.borrow()) {
            Ok(evaluator) => evaluator,
            Err(e) => {
                return Err(anyhow!("error while building the policy evaluator {:?}", e));
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
                return Err(anyhow!("error while evulating policy {:?}", e));
            }
        };
        if !result.allow {
            // since this datasource is not allowed by the group
            // let's drop the connection here.
            return Err(anyhow!(
                "incomming connection don't have access to the given datasource"
            ));
        }

        // terminate the connection if the incoming db access is fall under protected
        // attribute.
        if let Some(_) = result
            .protected_attributes
            .iter()
            .position(|attribute| attribute == params.get("database").unwrap())
        {
            return Err(anyhow!("unautorized db access"));
        }

        let mut handler = match ProtocolHandler::initialize(
            self.postgres_config.clone(),
            conn,
            params,
            self.policy_watcher.clone(),
            groups,
            evaluator,
            self.datasource.data_source_name.clone(),
            self.client.clone(),
            self.token.clone(),
            auth_res.passthrough,
            self.audit_sender.clone(),
        )
        .await
        {
            Ok(h) => h,
            Err(e) => {
                return Err(anyhow!("error while initializing protocol handler {:?}", e));
            }
        };
        handler.serve(auth_res.expires_at).await
    }

    // get_startup_msg returns the startup message and upgrade the connection to secure connection
    // if the client ask's for.
    async fn get_startup_msg(
        &self,
        mut conn: PostgresConn,
    ) -> Result<(FrontendMessage, PostgresConn), anyhow::Error> {
        loop {
            // decode the intial message form the client.
            let msg = match decode_init_startup_message(&mut conn).await {
                Ok(msg) => msg,
                Err(e) => {
                    match e {
                        DecoderError::UnsupporedVersion => {
                            return Err(anyhow!("unsupported postgres protocol version"))
                        }
                        _ => {
                            // log the error and close the connection.
                            return Err(anyhow!("error while decoding startup message {:?}", e));
                        }
                    };
                }
            };
            match msg {
                // sometimes client asks to upgrade the connection to tls. So, upgrade
                // before decoding the startup message.
                FrontendMessage::SslRequest => {
                    conn = self.upgrade_to_tls(conn).await?;
                    continue;
                }
                FrontendMessage::Startup { .. } => return Ok((msg, conn)),
                _ => return Err(anyhow!("invalid message")),
            }
        }
    }

    /// upgrade_to_tls will upgrade the given unsecured connection to secured connection.
    async fn upgrade_to_tls(&self, mut conn: PostgresConn) -> Result<PostgresConn, anyhow::Error> {
        if let None = self.ssl_acceptor {
            return Err(anyhow!(
                "don't have ssl acceptor to upgrade the connection to tls"
            ));
        }
        // upgrade the connection to tls only if the given connection is
        // insecured.
        if let PostgresConn::Unsecured(mut inner) = conn {
            if let Err(e) = inner.write_all(&[ACCEPT_SSL_ENCRYPTION]).await {
                return Err(anyhow!(
                    "error while sending ACCEPT_SSL_ENCRYPTION to client {:?}",
                    e
                ));
            }
            let ssl = Ssl::new(self.ssl_acceptor.as_ref().unwrap().context()).unwrap();
            let mut stream = SslStream::new(ssl, inner).unwrap();
            Pin::new(&mut stream).accept().await.unwrap();
            conn = PostgresConn::Secured(stream);
            return Ok(conn);
        }
        Err(anyhow!("can't upgrade secured connection"))
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

    /// get_call_opt return call option with control plane auth token. So that 
    /// it can used with grpc client while talking to
    fn get_call_opt(&self) -> CallOption {
        let mut meta_builder = grpcio::MetadataBuilder::new();
        meta_builder
            .add_str("auth-token", self.token.as_ref())
            .unwrap();
        let meta = meta_builder.build();
        return grpcio::CallOption::default().headers(meta);
    }
}
