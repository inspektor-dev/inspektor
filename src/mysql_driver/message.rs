use std::fmt::Debug;

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
use anyhow;
use byteorder::{ByteOrder, LittleEndian};
use bytes::{Bytes, BytesMut};
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub enum ClientMessage {
    RawMessage { seq_id: u8, payload: Bytes },
}

impl ClientMessage {
    pub async fn decode(stream: &mut TcpStream) -> Result<ClientMessage, anyhow::Error> {
        let mut buf = BytesMut::new();
        buf.resize(4, b'0');
        stream.read_exact(&mut buf).await?;
        // check the packet length.
        let packet_length = LittleEndian::read_u24(&buf) as usize;
        let seq_id = buf[3];
        buf.resize(packet_length, b'0');
        stream.read_exact(&mut buf).await?;
        return Ok(ClientMessage::RawMessage {
            seq_id,
            payload: buf.freeze(),
        });
    }
}

impl Debug for ClientMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RawMessage { seq_id, payload } => write!(
                f,
                "seq_id => {}, payload => {}",
                seq_id,
                String::from_utf8(payload.to_vec()).expect("Found invalid UTF-8")
            ),
        }
    }
}

pub enum ServerMessage {
    InitialHandShake {
        
    },
}
