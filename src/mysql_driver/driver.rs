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
use tokio;
use tokio::net::{TcpListener, TcpStream};
use anyhow::anyhow;
use log::*;

#[derive(Clone, Debug)]
pub struct MySqlDriver{}

impl MySqlDriver{
    pub fn start(&self) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let listener = TcpListener::bind(format!(
                "0.0.0.0:{}",
                2000
            ))
            .await
            .map_err(|_| anyhow!("unable to listern on the given port"))
            .unwrap();
            info!(
                "mysql driver listeneing at 0.0.0.0:{}",
                2000
            );
            loop {
                let (socket, _) = listener.accept().await.unwrap();
                //       let acceptor = acceptor.clone();
                let driver = self.clone();
                tokio::spawn(async move {
                    if let Err(e) = driver.handle_client_conn(socket).await {
                        error!("error while handling client connection {:?}", e);
                    }
                    ()
                });
            }
        }); 
    }

    async fn handle_client_conn(&self, conn: TcpStream) -> Result<(), anyhow::Error>{
        // to initate the handshake, we must get the target mysql configuration.
        
        todo!()
    }
}