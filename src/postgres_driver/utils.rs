use crate::postgres_driver::errors::DecoderError;
use crate::postgres_driver::message::*;
use anyhow::*;
use byteorder::{ByteOrder, NetworkEndian};
use bytes::{Buf, BufMut, BytesMut};
use std::char;
use std::collections::HashMap;
use std::io::ErrorKind;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
// Postgres protocol version.

// decode_startup_message decode pg startup message, if ssl request it'll  upgrade the connection to ssl connection and returns the
pub async fn decode_init_startup_message<T>(mut conn: T) -> Result<FrotendMessage, DecoderError>
where
    T: AsyncRead + Unpin + AsyncReadExt + AsyncWrite + AsyncWriteExt,
{
    // read the frame length.
    let len = decode_frame_length(&mut conn).await?;
    let mut buf = BytesMut::new();
    buf.resize(len, b'0');
    conn.read_exact(&mut buf).await?;
    // read version number
    let version_number = buf.get_i32();
    match version_number {
        VERSION_SSL => return Ok(FrotendMessage::SslRequest),
        VERSION_3 => {
            let mut params = HashMap::new();
            // read all the params.
            // have to make it safe.
            while *buf.get(0).unwrap() != 0 {
                let key =
                    read_cstr(&mut buf).map_err(|_| anyhow!("error while reading key params"))?;
                let val =
                    read_cstr(&mut buf).map_err(|_| anyhow!("error while reading value params"))?;
                params.insert(key, val);
            }

            return Ok(FrotendMessage::Startup {
                params: params,
                version: version_number,
            });
        }
        _ => {
            return Err(DecoderError::UnsupporedVersion);
        }
    };
}

pub async fn decode_frame_length<T>(mut conn: T) -> Result<usize, anyhow::Error>
where
    T: AsyncRead + Unpin,
{
    let mut buf = [0; 4];
    conn.read_exact(&mut buf).await?;
    let frame_len = NetworkEndian::read_u32(&buf) as usize;
    if frame_len < 4 {
        // client didn't include the length of frame length itself.
        return Err(anyhow!("invalid frame length"));
    }
    Ok(frame_len - 4)
}

pub fn read_cstr(buf: &mut BytesMut) -> Result<String, Error> {
    if let Some(pos) = buf.iter().position(|d| *d == 0) {
        let str = std::str::from_utf8(&buf[..pos])
            .map_err(|_| anyhow!("error while reading cstr"))?
            .to_string();
        buf.advance(pos + 1);
        return Ok(str);
    }
    Err(anyhow!("string has not termination deliminiter"))
}

pub fn write_cstr(buf: &mut BytesMut, val: &[u8]) -> Result<(), anyhow::Error> {
    if val.contains(&0) {
        return Err(anyhow!("cstr should not contain 0 value"));
    }
    buf.put_slice(val);
    buf.put_u8(0);
    Ok(())
}

pub async fn decode_password_message<T>(mut conn: T) -> Result<FrotendMessage, anyhow::Error>
where
    T: AsyncRead + AsyncReadExt + Unpin,
{
    let mut buf = [0; 1];
    // loop till it read it.
    conn.read_exact(&mut buf).await?;

    if buf[0] != b'p' {
        return Err(anyhow!("incoming message is not a password message"));
    }
    let len = decode_frame_length(&mut conn).await.map_err(|_| {
        anyhow!("error while decoding frame length while decoding password message")
    })?;
    let mut buf = BytesMut::new();
    buf.resize(len, b'0');
    conn.read_exact(&mut buf).await?;
    // read the passcode.
    let password =
        read_cstr(&mut buf).map_err(|err| anyhow!("error while reading password {:?}", err))?;
    Ok(FrotendMessage::PasswordMessage { password: password })
}

#[inline]
pub fn write_message<F>(buf: &mut BytesMut, f: F) -> Result<(), anyhow::Error>
where
    F: FnOnce(&mut BytesMut) -> Result<(), anyhow::Error>,
{
    let base = buf.len();
    buf.extend_from_slice(&[0; 4]);

    f(buf)?;

    let size = (buf.len() - base) as i32;
    NetworkEndian::write_i32(&mut buf[base..], size);
    Ok(())
}

#[inline]
pub fn read_counted_message<F, T>(buf: &mut BytesMut,mut f: F) -> Result<Vec<T>, anyhow::Error>
where
    F: FnMut(&mut BytesMut) -> Result<T, anyhow::Error>,
{
    let len = NetworkEndian::read_i16(buf) as usize;
    buf.advance(2);
    let mut result = Vec::with_capacity(len);
    for _ in 0..len {
        result.push(f(buf)?);
    }
    Ok(result)
}

#[inline]
pub fn write_counted_message<I, T, F>(items: I, mut f: F, buf: &mut BytesMut) -> Result<(), anyhow::Error>
where
    I: IntoIterator<Item = T>,
    F: FnMut(T, &mut BytesMut) -> Result<(), anyhow::Error>,
{
    let base = buf.len();
    buf.extend_from_slice(&[0; 2]);
    let mut count = 0;
    for item in items {
        f(item, buf)?;
        count += 1;
    }
    let count = count as i16;
    NetworkEndian::write_i16(&mut buf[base..], count);
    Ok(())
}