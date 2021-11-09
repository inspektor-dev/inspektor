use bytes::{BytesMut, BufMut};
use std::collections::HashMap;

#[derive(Debug)]
pub enum StartupMessage {
    Startup {
        params: HashMap<String, String>,
        version: i32,
    },
    AuthenticationCleartextPassword,
    PasswordMessage{
        password: String,
    },
    AuthenticationOk
}

impl StartupMessage {
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        match self{
            StartupMessage::AuthenticationCleartextPassword =>{
                buf.put_u8(b'R');
                buf.put_u32(8);
                buf.put_u32(3);
                buf
            },
            StartupMessage::AuthenticationOk =>{
                buf.put_u8(b'R');
                buf.put_u32(8);
                buf.put_u32(2);
                buf
            },
            _ => {
                unreachable!("encoding invalid startup message")
            }
        }
    }
}

#[derive(Debug)]
pub enum BackendMessage {}
