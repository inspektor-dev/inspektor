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
use log::info;
use async_trait::async_trait;
/// StdOutLogs is responsible for printing audit logs to the std out
/// with a specified prefix.
pub struct StdOutLogs {
    log_prefix: String,
}

impl StdOutLogs {
    pub fn new(log_prefix: String) -> StdOutLogs {
        return StdOutLogs{
            log_prefix
        }
    }
}

#[async_trait]
impl AuditLog for StdOutLogs {
    async fn push_log(&mut self, log: String) {
        info!("[{}] {}", self.log_prefix, log)
    }
}