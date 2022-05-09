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
use super::AuditLog;
use crate::apiproto::api::CloudWatchConfig;
use anyhow;
use async_trait::async_trait;
use aws_config;
use aws_sdk_cloudwatchlogs::model::InputLogEvent;
use log::*;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

/// CloudWatchLogs is reponsible for sending audit logs to
/// cloud watch logs.
pub struct CloudWatchLogs {
    client: aws_sdk_cloudwatchlogs::Client,
    cloud_watch_config: CloudWatchConfig,
    next_token: Option<String>,
}

impl CloudWatchLogs {
    /// new will return cloud watch client
    pub async fn new(config: CloudWatchConfig) -> Result<CloudWatchLogs, anyhow::Error> {
        env::set_var("AWS_DEFAULT_REGION", config.region_name.clone());
        if config.get_cred_type() == "cred" {
            env::set_var("AWS_ACCESS_KEY_ID", config.access_key.clone());
            env::set_var("AWS_SECRET_ACCESS_KEY", config.secret_key.clone());
        }
        let sdk_config = aws_config::load_from_env().await;
        let client = aws_sdk_cloudwatchlogs::Client::new(&sdk_config);
        Ok(CloudWatchLogs {
            client: client,
            cloud_watch_config: config,
            next_token: None,
        })
    }

    /// set_next_token get sequence token from aws and set's it to push new logs.
    pub async fn set_next_token(&mut self) -> Result<(), anyhow::Error> {
        let describe_log_group = self.client.describe_log_streams();
        let res = describe_log_group
            .set_log_group_name(Some(
                self.cloud_watch_config
                    .get_log_group_name()
                    .clone()
                    .to_string(),
            ))
            .set_log_stream_name_prefix(Some(
                self.cloud_watch_config
                    .get_log_stream_name()
                    .clone()
                    .to_string(),
            ))
            .send()
            .await?;
        self.next_token = res.next_token;
        Ok(())
    }
}

#[async_trait]
impl AuditLog for CloudWatchLogs {
    async fn push_log(&mut self, log: String) {
        self.set_next_token().await.unwrap();
        let now = SystemTime::now();
        let events = InputLogEvent::builder()
            .set_message(Some(log))
            .set_timestamp(Some(
                now.duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
            ))
            .build();

        let req = self
            .client
            .put_log_events()
            .set_log_group_name(Some(
                self.cloud_watch_config
                    .get_log_group_name()
                    .clone()
                    .to_string(),
            ))
            .set_log_stream_name(Some(
                self.cloud_watch_config
                    .get_log_stream_name()
                    .clone()
                    .to_string(),
            ))
            .set_sequence_token(self.next_token.clone());
        match req.log_events(events).send().await {
            Ok(res) => {
                self.next_token = res.next_sequence_token;
            }
            Err(e) => {
                error!("error while sending logs to cloudwatch {:?}", e);
            }
        }
    }
}
