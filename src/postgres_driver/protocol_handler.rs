// Copyright 2021 Balaji (rbalajis25@gmail.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use crate::config::PostgresConfig;
use crate::policy_evaluator::evaluator::PolicyEvaluator;
use crate::postgres_driver::conn::PostgresConn;
use crate::postgres_driver::errors::ProtocolHandlerError;
use crate::postgres_driver::message::*;
use crate::sql::ctx::Ctx;
use crate::sql::query_rewriter::QueryRewriter;
use anyhow::*;
use log::*;
use md5::{Digest, Md5};
use openssl::ssl::{Ssl, SslConnector, SslMethod};
use postgres_protocol::authentication::sasl;
use std::collections::{HashMap, HashSet};
use std::pin::Pin;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::time as tokio_time;
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
    policy_evaluator: PolicyEvaluator,
    groups: Vec<String>,
    config: PostgresConfig,
    connected_db: String,
    datasource_name: String,
}

#[derive(Default)]
struct TableInfo {
    column_relation: HashMap<String, Vec<String>>,
    schemas: Vec<String>,
}

impl ProtocolHandler {
    // get_table_info get table info of the protected tables.
    async fn get_table_info(
        &mut self,
        client: &tokio_postgres::Client,
    ) -> Result<TableInfo, anyhow::Error> {
        let result = self.policy_evaluator.evaluate(
            &self.datasource_name,
            &self.connected_db,
            &self.groups,
        )?;
        let protected_tables = result.get_protected_tables();

        if protected_tables.len() == 0 {
            return Ok(TableInfo::default());
        }

        // query rewriter needs only the table info of the protected table, so
        // query only neccessary info.
        let mut schema_selection = String::from("(");
        let mut table_selection = String::from("(");
        let mut delim = "";
        for protected_table in protected_tables {
            schema_selection.push_str(delim);
            table_selection.push_str(delim);
            delim = ",";
            schema_selection.push('\'');
            schema_selection.push_str(protected_table.0);
            schema_selection.push('\'');
            table_selection.push('\'');
            table_selection.push_str(protected_table.1);
            table_selection.push('\'');
        }
        schema_selection.push(')');
        table_selection.push(')');
        let query = format!(
            r#"
        SELECT 
          table_schema, 
          table_name, 
          column_name, 
          data_type 
        FROM 
          information_schema.columns 
        where 
          table_schema in {}
          and table_name in {}
        "#,
            schema_selection, table_selection
        );

        let rows = client.query(&query, &[]).await?;

        let mut column_relation: HashMap<String, Vec<String>> = HashMap::default();
        let mut schemas: HashSet<String> = HashSet::default();
        for row in rows {
            let schema_name = row.get::<usize, String>(0);
            // table name is format of both schema and table.
            let table_name: String = format!("{}.{}", &schema_name, row.get::<usize, String>(1));
            if !schemas.contains(&schema_name) {
                schemas.insert(schema_name);
            }
            let column_name: String = row.get(2);
            if let Some(columns) = column_relation.get_mut(&table_name) {
                columns.push(column_name);
                continue;
            }
            column_relation.insert(table_name, vec![column_name]);
        }
        let schemas = schemas.into_iter().collect::<Vec<_>>();
        Ok(TableInfo {
            column_relation,
            schemas,
        })
    }

    // serve will listen to client packets and decide whether to process
    // the packet based on the opa policy.
    pub async fn serve(&mut self) -> Result<(), anyhow::Error> {
        debug!("started serving");

        let (client, connection) = tokio_postgres::connect(
            &format!(
                "host=localhost port=5432 user={} dbname = {} password = {}",
                self.config.target_username.as_ref().unwrap(),
                self.connected_db,
                self.config.target_password.as_ref().unwrap()
            ),
            tokio_postgres::NoTls,
        )
        .await?;
        tokio::spawn(connection);
        let mut table_info = self.get_table_info(&client).await.map_err(|e| {
            error!("error while getting table meta {:?}", e);
            return anyhow!("error while getting table meta");
        })?;
        let mut target_buf = [0; 1024];
        // refresh table for every 2 minutes.
        let mut table_info_refresh_ticker = tokio_time::interval(Duration::from_secs(60 * 2));
        loop {
            tokio::select! {
                evaluator = self.policy_watcher.changed() => {
                    if !evaluator.is_ok(){
                        error!("watched failed to get new evaluation. prolly watcher closed");
                        continue;
                    }
                    let wasm_policy = self.policy_watcher.borrow();
                    // update the current evaluator with new policy
                    let mut evaluator = match PolicyEvaluator::new(&wasm_policy){
                        Ok(evaluator) => evaluator,
                        Err(_) => {
                            error!("error while building new policy evaluator so skiping this policy.");
                            continue;
                        }
                    };
                    // let's check whether new policy allows the current db connection
                    let result = evaluator.evaluate(&self.datasource_name, &self.connected_db, &self.groups)?;
                    if !result.allow{
                        error!("updated policy violating the existing connection so dropping the connection");
                        continue;
                    }
                    self.policy_evaluator = evaluator;
                }
                n = self.target_conn.read(&mut target_buf) => {
                    match n {
                        Err(e) =>{
                                println!("failed to read from socket; err = {:?}", e);
                                return Ok(());
                        },
                        Ok(n) =>{
                            if n == 0 {
                               return Ok(());
                            }
                            if let Err(e) = self.client_conn.write_all(&target_buf[0..n]).await{
                                error!("error while writing the rsp message to the client {:?}", e);
                                return Ok(());
                            }
                        }
                    }
                }
                n = FrontendMessage::decode(&mut self.client_conn) => {
                    match n {
                        Err(e) =>{
                                println!("failed to read from socket; err = {:?}", e);
                                return Ok(());
                        },
                        Ok(mut msg) =>{
                            debug!("got frontend message {:?}", msg);
                            let ctx =  Ctx::new(table_info.column_relation.clone());
                            if let Err(e) = self.handle_frontend_message(&mut msg, ctx, table_info.schemas.clone()){
                                error!("error while handling frontend message {:?}", e);
                                let rsp = BackendMessage::ErrorMsg(Some(format!("{:?}", e)));
                                if let Err(e) = self.client_conn.write_all(&rsp.encode()).await{
                                    error!("error while writing the rsp message to the client {:?}", e);
                                    return Ok(());
                                }
                                continue;
                            }
                            if let Err(e) = self.target_conn.write_all(&msg.encode()).await{
                                error!("error while writing the frontend message to the target {:?}", e);
                                return Ok(());
                            }
                        }
                    }
                }
                _ = table_info_refresh_ticker.tick() => {
                    debug!("refreshing table meta");
                    table_info = match  self.get_table_info(&client).await {
                        Ok(info) => info,
                        Err(e) => {
                            error!("error while refreshing table meta {:?}", e);
                            continue;
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
        groups: Vec<String>,
        evaluator: PolicyEvaluator,
        datasource_name: String,
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
                BackendMessage::AuthenticationSASL { mechanisms } => {
                    debug!(
                        "sasl authhentication requested with the following mechanism {:?}",
                        mechanisms
                    );

                    if mechanisms
                        .iter()
                        .position(|mechanism| *mechanism == "SCRAM-SHA-256")
                        .is_none()
                    {
                        return Err(anyhow!(
                            "supported sasl mechanism is SCRAM-SHA-256. but requested for {:?}",
                            mechanisms
                        ));
                    }
                    ProtocolHandler::authenticate_sasl(
                        &mut target_conn,
                        config.target_password.as_ref().unwrap(),
                    )
                    .await?;
                    debug!("sasl authentication completed successfully");
                    continue;
                }
                BackendMessage::AuthenticationOk { .. } => {
                    // send authentication ok to client connection since we established connection with
                    // target.
                    client_conn.write_all(&rsp_msg.encode()).await?;
                    let handler = ProtocolHandler {
                        target_conn: target_conn,
                        client_conn: client_conn,
                        policy_watcher: policy_watcher,
                        policy_evaluator: evaluator,
                        groups: groups,
                        config: config.clone(),
                        connected_db: client_parms.get("database").unwrap().clone(),
                        datasource_name: datasource_name,
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

    async fn authenticate_sasl(
        mut target_conn: &mut PostgresConn,
        password: &String,
    ) -> Result<(), anyhow::Error> {
        // refer: https://datatracker.ietf.org/doc/html/rfc5802 for more context.
        let pass_buf = password.as_bytes();
        let mut sasl_auth = sasl::ScramSha256::new(pass_buf, sasl::ChannelBinding::unsupported());
        debug!("sending client first message");
        // send client first message to the target.
        target_conn
            .write_all(
                &FrontendMessage::SASLInitialResponse {
                    body: sasl_auth.message().to_vec(),
                    mechanism: String::from("SCRAM-SHA-256"),
                }
                .encode(),
            )
            .await?;
        debug!("receiving server first message");
        // get server first message form the target.
        let msg = decode_backend_message(&mut target_conn).await?;
        let data = match msg {
            BackendMessage::AuthenticationSASLContinue { data } => data,
            _ => {
                error!("expected sasl continue message but got {:?}", msg);
                return Err(anyhow!("expected sasl continue message"));
            }
        };
        println!("data from {:?}", String::from_utf8(data.clone()).unwrap());
        sasl_auth.update(&data[..]).map_err(|e| {
            error!("error while updating server first message {:?}", e);
            e
        })?;
        debug!("sending client final message");
        // send client final message.
        target_conn
            .write_all(
                &FrontendMessage::SASLResponse {
                    body: sasl_auth.message().to_vec(),
                }
                .encode(),
            )
            .await?;
        // retrive server final message and verify.
        debug!("receiving server final message");
        let msg = decode_backend_message(&mut target_conn).await?;
        let data = match msg {
            BackendMessage::AuthenticationSASLFinal { data } => data,
            _ => {
                error!("expected sasl final message but got {:?}", msg);
                return Err(anyhow!("expected sasl final message"));
            }
        };
        sasl_auth.finish(&data[..])?;
        Ok(())
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
                let connector = ProtocolHandler::get_ssl_connector();
                let mut stream = SslStream::new(connector, inner).unwrap();
                Pin::new(&mut stream).connect().await.map_err(|e| {
                    error!(
                        "unable to upgrade the target connection to ssl stream {:?}",
                        e
                    );
                    anyhow!("error while upgrading target connection to ssl stream")
                })?;
                debug!("taget connection upgraded to tls connection");
                Ok(PostgresConn::Secured(stream))
            }
            _ => Ok(conn),
        }
    }

    fn get_ssl_connector() -> Ssl {
        SslConnector::builder(SslMethod::tls())
            .unwrap()
            .build()
            .configure()
            .unwrap()
            .verify_hostname(false)
            .use_server_name_indication(false)
            .into_ssl("")
            .unwrap()
    }

    fn handle_frontend_message(
        &mut self,
        msg: &mut FrontendMessage,
        ctx: Ctx,
        schemas: Vec<String>,
    ) -> Result<(), ProtocolHandlerError> {
        match msg {
            FrontendMessage::Query { query_string } => {
                self.handle_query(query_string, ctx, schemas)?;
            }
            FrontendMessage::Parse { query, .. } => {
                self.handle_query(query, ctx, schemas)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_query(
        &mut self,
        query: &mut String,
        ctx: Ctx,
        schemas: Vec<String>,
    ) -> Result<(), ProtocolHandlerError> {
        // TODO: this needs to worked out.
        if query.contains("BEGIN READ ONLY") {
            return Ok(());
        }
        let result = self.policy_evaluator.evaluate(
            &self.datasource_name,
            &self.connected_db,
            &self.groups,
        )?;
        if !result.allow {
            error!("updated policy violating the existing connection so dropping the connection");
            return Err(ProtocolHandlerError::PolicyRejected);
        }
        let dialect = sqlparser::dialect::PostgreSqlDialect {};
        let mut statements = match sqlparser::parser::Parser::parse_sql(&dialect, query) {
            Ok(statements) => statements,
            Err(e) => {
                error!(
                    "error while parsing user query error: {} query string: {}",
                    e, query
                );
                return Err(ProtocolHandlerError::ErrParsingQuery);
            }
        };
        let rule = result.to_rule_engine();
        let rewriter = QueryRewriter::new(rule, schemas);
        rewriter.rewrite(&mut statements, ctx)?;
        let mut out = String::from("");
        for statement in statements {
            out = format!("{}{};", out, statement);
        }
        *query = out;
        Ok(())
    }
}
