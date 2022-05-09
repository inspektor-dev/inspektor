// Copyright 2022 poonai
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

mod cloudwatch;
use crate::apiproto::api::CloudWatchConfig;
use crate::auditlog::cloudwatch::CloudWatchLogs;
use async_trait::async_trait;
use std::thread;
use tokio::sync::mpsc;

#[async_trait]
pub trait AuditLog {
    async fn push_log(&mut self, log: String);
}

/// start_audit_worker will start the audit log worker. it listens for audit logs from
/// data source driver and push the audit logs to the configured audit log destination.
pub fn start_audit_worker(cfg: CloudWatchConfig) -> mpsc::Sender<String> {
    // mpsc channels are used communicate between audit worker and from the
    // audit producer.
    let (tx, mut rx) = mpsc::channel(32);
    thread::spawn(move || {
        // run the worker in different thread to listen for audit logs.
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // get the respective audit client.
            let mut audit_client: Option<Box<dyn AuditLog + Send>> = None;
            if cfg.get_cred_type() != "" {
                audit_client = Some(Box::new(
                    CloudWatchLogs::new(cfg)
                        .await
                        .expect("check cloud watch config"),
                ));
            }
            loop {
                let log = rx.recv().await.unwrap();
                if audit_client.is_none() {
                    continue;
                }
                unsafe {
                    audit_client.as_mut().unwrap_unchecked().push_log(log).await;
                }
            }
        })
    });
    tx
}
