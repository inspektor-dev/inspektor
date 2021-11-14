use crate::postgres_driver::codec::{decode_frame_length, read_cstr, write_cstr, write_message};
use anyhow::*;
use byteorder::{ByteOrder, NetworkEndian};
use bytes::{Buf, BufMut, BytesMut};
use log::*;
use std::collections::HashMap;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const VERSION_3: i32 = 0x30000;
pub const VERSION_SSL: i32 = (1234 << 16) + 5679;
pub const ACCEPT_SSL_ENCRYPTION: u8 = b'S';

#[derive(Debug)]
pub enum StartupMessage {
    Startup {
        params: HashMap<String, String>,
        version: i32,
    },
    AuthenticationCleartextPassword,
    PasswordMessage {
        password: String,
    },
    AuthenticationOk {
        success: bool,
    },
    SslRequest,
    AuthenticationMD5Password {
        salt: Vec<u8>,
    },
}

impl StartupMessage {
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        match self {
            StartupMessage::AuthenticationCleartextPassword => {
                buf.put_u8(b'R');
                buf.put_u32(8);
                buf.put_u32(3);
                buf
            }
            StartupMessage::AuthenticationOk { success } => {
                buf.put_u8(b'R');
                buf.put_u32(8);
                if *success {
                    buf.put_u32(0);
                    return buf
                }
                buf.put_u32(1);
                buf
            }
            StartupMessage::SslRequest => {
                buf.put_u32(8);
                buf.put_i32(VERSION_SSL);
                buf
            }
            StartupMessage::Startup { params, version } => {
                write_message(&mut buf, |buf| {
                    buf.put_i32(*version);
                    for (key, val) in params {
                        write_cstr(buf, key.as_bytes())?;
                        write_cstr(buf, val.as_bytes())?;
                    }
                    buf.put_u8(0);
                    Ok(())
                })
                .unwrap();
                buf
            }
            _ => {
                unreachable!("encoding invalid startup message")
            }
        }
    }
}
#[derive(Debug)]
pub enum BackendMessage {
    StartupMessage(StartupMessage),
    ErrorMsg(Option<String>),
}

pub async fn decode_backend_message<T>(mut conn: T) -> Result<BackendMessage, anyhow::Error>
where
    T: AsyncRead + Unpin + AsyncReadExt + AsyncWrite + AsyncWriteExt,
{
    // read the first byte.
    let mut meta = [0; 1];
    conn.read_exact(&mut meta).await.map_err(|e| {
        error!("error while reading the meta for backend message {:?}", e);
        anyhow!("error reading backend meta")
    })?;

    match meta[0] {
        b'R' => {
            let len = decode_frame_length(&mut conn).await.map_err(|e| {
                error!("error while decoding frame lenght {:?}", e);
                anyhow!("invalid backend message")
            })?;
            let mut buf = BytesMut::new();
            buf.resize(len, b'0');
            conn.read_exact(&mut buf).await.map_err(|err| {
                error!("error while reading backend message {:?}", err);
                anyhow!("error while decoding error messaage")
            })?;

            let msg_type = NetworkEndian::read_u32(&buf);
            buf.advance(4);
            match msg_type {
                3 => {
                    return Ok(BackendMessage::StartupMessage(
                        StartupMessage::AuthenticationCleartextPassword,
                    ))
                }
                5 => {
                    let salt = buf[..4].to_vec();
                    return Ok(BackendMessage::StartupMessage(
                        StartupMessage::AuthenticationMD5Password { salt },
                    ));
                }
                0 => {
                    return Ok(BackendMessage::StartupMessage(
                        StartupMessage::AuthenticationOk{success: true},
                    ));
                }
                1 => {
                    return Ok(BackendMessage::StartupMessage(
                        StartupMessage::AuthenticationOk{success: true},
                    ));
                }
                _ => {
                    unreachable!("unknown message type {:?}", msg_type)
                }
            }
        }
        b'E' => {
            let msg_len = decode_frame_length(&mut conn).await?;
            let mut buf = BytesMut::new();
            buf.resize(msg_len, b'0');
            conn.read_exact(&mut buf)
                .await
                .map_err(|_| anyhow!("error while decoing err msg from postgres target"))?;
            if buf[0] == 0 {
                return Ok(BackendMessage::ErrorMsg(None));
            }
            buf.advance(1);
            let err_msg = read_cstr(&mut buf)?;
            return Ok(BackendMessage::ErrorMsg(Some(err_msg)));
        }
        _ => {
            error!("invalid meta message for backend {:?}", meta[0]);
            return Err(anyhow!("invalid backend message"));
        }
    }

    unreachable!("");
}

#[derive(Debug)]
pub enum FrotendMessage {
    PasswordMessage { password: String },
}

impl FrotendMessage {
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        match self {
            FrotendMessage::PasswordMessage { password } => {
                buf.put_u8(b'p');
                write_message(&mut buf, |buf| {
                    write_cstr(buf, password.as_bytes())?;
                    Ok(())
                })
                .unwrap();
            }
        }
        buf
    }
}
