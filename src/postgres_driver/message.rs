use crate::postgres_driver::utils::{
    decode_frame_length, read_counted_message, read_cstr, write_counted_message, write_cstr,
    write_message,
};
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
pub enum Value {
    Null,
    NotNull(Vec<u8>),
}

#[derive(Debug)]
pub enum ReadyState {
    Idle,
    Transaction,
    FailedTransaction,
}

impl ReadyState {
    fn get_state_byte(&self) -> u8 {
        match self {
            ReadyState::Idle => b'I',
            ReadyState::Transaction => b'T',
            ReadyState::FailedTransaction => b'E',
        }
    }
}

#[derive(Debug)]
pub enum BackendMessage {
    ErrorMsg(Option<String>),
    AuthenticationOk { success: bool },
    AuthenticationCleartextPassword,
    AuthenticationMD5Password { salt: Vec<u8> },
    AuthenticationSASL { mechanisms: Vec<String> },
    AuthenticationSASLContinue { data: Vec<u8> },
    AuthenticationSASLFinal { data: Vec<u8> },
    ReadyForQuery { state: ReadyState },
}

impl BackendMessage {
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        match self {
            BackendMessage::AuthenticationCleartextPassword => {
                buf.put_u8(b'R');
                buf.put_u32(8);
                buf.put_u32(3);
                buf
            }
            BackendMessage::AuthenticationOk { success } => {
                buf.put_u8(b'R');
                buf.put_u32(8);
                if *success {
                    buf.put_u32(0);
                    return buf;
                }
                buf.put_u32(1);
                buf
            }
            BackendMessage::ErrorMsg(msg) => {
                buf.put_u8(b'E');
                write_message(&mut buf, |buf| {
                    if let Some(msg) = msg {
                        buf.put_u8(b'S');
                        write_cstr(buf, "ERROR".to_string().as_bytes())?;
                        buf.put_u8(b'C');
                        write_cstr(buf, "42501".to_string().as_bytes())?;
                        buf.put_u8(b'M');
                        write_cstr(buf, msg.as_bytes())?;
                    }
                    buf.put_u8(b'\0');
                    Ok(())
                })
                .unwrap();
                buf
            }
            BackendMessage::ReadyForQuery { state } => {
                buf.put_u8(b'Z');
                buf.put_u32(5);
                buf.put_u8(state.get_state_byte());
                buf
            }
            _ => {
                unreachable!("encoding invalid startup message")
            }
        }
    }
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
                3 => return Ok(BackendMessage::AuthenticationCleartextPassword),
                5 => {
                    let salt = buf[..4].to_vec();
                    return Ok(BackendMessage::AuthenticationMD5Password { salt });
                }
                0 => {
                    return Ok(BackendMessage::AuthenticationOk { success: true });
                }
                1 => {
                    return Ok(BackendMessage::AuthenticationOk { success: false });
                }
                10 => {
                    let mut mechanisms = Vec::new();
                    while *buf.get(0).unwrap() != 0 {
                        mechanisms.push(read_cstr(&mut buf)?);
                    }
                    return Ok(BackendMessage::AuthenticationSASL {
                        mechanisms: mechanisms,
                    });
                }
                11 => return Ok(BackendMessage::AuthenticationSASLContinue { data: buf.to_vec() }),
                12 => return Ok(BackendMessage::AuthenticationSASLFinal { data: buf.to_vec() }),
                _ => {
                    unreachable!("unknown message type {:?}", msg_type)
                }
            }
        }
        b'E' => {
            let msg_len = decode_frame_length(&mut conn).await.map_err(|e| {
                error!("error while decoding backend error message length");
                e
            })?;
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
}

#[derive(Debug)]
pub enum FrontendMessage {
    PasswordMessage {
        password: String,
    },
    SslRequest,
    Startup {
        params: HashMap<String, String>,
        version: i32,
    },
    Describe {
        is_prepared_statement: bool,
        name: String,
    },
    Flush,
    Query {
        query_string: String,
    },
    Sync,
    Terminate,
    Bind {
        destination_portal_name: String,
        prepared_statement_name: String,
        parameter_format_codes: Vec<i16>,
        parameter_values: Vec<Value>,
        result_column_format_codes: Vec<i16>,
    },
    Close {
        is_portal: bool,
        name: String,
    },
    CopyData(Vec<u8>),
    CopyDone,
    CopyFail {
        err_msg: String,
    },
    Execute {
        name: String,
        max_no_of_rows: i32,
    },
    FunctionCall {
        object_id: i32,
        format_codes: Vec<i16>,
        function_arguments: Vec<Value>,
        result_format_code: i16,
    },
    Parse {
        name: String,
        query: String,
        object_ids: Vec<i32>,
    },
    SASLInitialResponse {
        mechanism: String,
        body: Vec<u8>,
    },
    SASLResponse {
        body: Vec<u8>,
    },
}

impl FrontendMessage {
    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        match self {
            FrontendMessage::PasswordMessage { password } => {
                buf.put_u8(b'p');
                write_message(&mut buf, |buf| {
                    write_cstr(buf, password.as_bytes())?;
                    Ok(())
                })
                .unwrap();
            }
            FrontendMessage::SslRequest => {
                buf.put_u32(8);
                buf.put_i32(VERSION_SSL);
            }
            FrontendMessage::Startup { params, version } => {
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
            }
            FrontendMessage::Describe {
                is_prepared_statement,
                name,
            } => {
                buf.put_u8(b'D');
                write_message(&mut buf, |buf| {
                    if *is_prepared_statement {
                        buf.put_u8(b'S');
                    } else {
                        buf.put_u8(b'P');
                    }
                    if name.len() != 0 {
                        write_cstr(buf, name.as_bytes())?;
                    } else {
                        buf.put_u8(0);
                    }
                    Ok(())
                })
                .unwrap();
            }
            FrontendMessage::Flush => {
                buf.put_u8(b'H');
                buf.put_i32(4);
            }
            FrontendMessage::Query { query_string } => {
                buf.put_u8(b'Q');
                write_message(&mut buf, |buf| write_cstr(buf, query_string.as_bytes())).unwrap();
            }
            FrontendMessage::Sync => {
                buf.put_u8(b'S');
                buf.put_i32(4);
            }
            FrontendMessage::Terminate => {
                buf.put_u8(b'X');
                buf.put_i32(4);
            }
            FrontendMessage::Bind {
                destination_portal_name,
                prepared_statement_name,
                parameter_format_codes,
                parameter_values,
                result_column_format_codes,
            } => {
                buf.put_u8(b'B');
                write_message(&mut buf, |buf| {
                    write_cstr(buf, destination_portal_name.as_bytes()).unwrap();
                    write_cstr(buf, prepared_statement_name.as_bytes()).unwrap();
                    write_counted_message(
                        parameter_format_codes,
                        |item, buf| {
                            buf.put_i16(*item);
                            Ok(())
                        },
                        buf,
                    )?;
                    write_counted_message(
                        parameter_values,
                        |item, buf| {
                            match item {
                                Value::Null => {
                                    buf.put_i32(-1);
                                }
                                Value::NotNull(val) => {
                                    buf.put_i32(val.len() as i32);
                                    if val.len() != 0 {
                                        buf.extend_from_slice(val);
                                    }
                                }
                            }
                            Ok(())
                        },
                        buf,
                    )?;
                    write_counted_message(
                        result_column_format_codes,
                        |item, buf| {
                            buf.put_i16(*item);
                            Ok(())
                        },
                        buf,
                    )?;
                    Ok(())
                })
                .unwrap();
            }
            FrontendMessage::FunctionCall {
                object_id,
                format_codes,
                function_arguments,
                result_format_code,
            } => {
                write_message(&mut buf, |buf| {
                    buf.put_i32(*object_id);
                    write_counted_message(
                        format_codes,
                        |item, buf| {
                            buf.put_i16(*item);
                            Ok(())
                        },
                        buf,
                    )
                    .unwrap();
                    write_counted_message(
                        function_arguments,
                        |item, buf| {
                            match item {
                                Value::Null => {
                                    buf.put_i32(-1);
                                }
                                Value::NotNull(val) => {
                                    buf.put_i32(val.len() as i32);
                                    if val.len() != 0 {
                                        buf.extend_from_slice(val);
                                    }
                                }
                            }
                            Ok(())
                        },
                        buf,
                    )
                    .unwrap();
                    buf.put_i16(*result_format_code);
                    Ok(())
                })
                .unwrap();
            }
            FrontendMessage::CopyData(data) => {
                buf.put_u8(b'd');
                write_message(&mut buf, |buf| {
                    buf.extend_from_slice(data);
                    Ok(())
                })
                .unwrap();
            }
            FrontendMessage::CopyDone => {
                buf.put_u8(b'c');
                write_message(&mut buf, |_| Ok(())).unwrap();
            }
            FrontendMessage::CopyFail { err_msg } => {
                buf.put_u8(b'f');
                write_message(&mut buf, |buf| Ok(write_cstr(buf, err_msg.as_bytes())?)).unwrap();
            }
            FrontendMessage::Close { is_portal, name } => {
                buf.put_u8(b'C');
                write_message(&mut buf, |buf| {
                    if *is_portal {
                        buf.put_u8(b'P');
                    } else {
                        buf.put_u8(b'S');
                    }
                    write_cstr(buf, name.as_bytes())
                })
                .unwrap();
            }
            FrontendMessage::Execute {
                name,
                max_no_of_rows,
            } => {
                buf.put_u8(b'E');
                write_message(&mut buf, |buf| {
                    write_cstr(buf, name.as_bytes())?;
                    buf.put_i32(*max_no_of_rows);
                    Ok(())
                })
                .unwrap();
            }
            FrontendMessage::Parse {
                name,
                query,
                object_ids,
            } => {
                buf.put_u8(b'P');
                write_message(&mut buf, |buf| {
                    write_cstr(buf, name.as_bytes())?;
                    write_cstr(buf, query.as_bytes())?;
                    write_counted_message(
                        object_ids,
                        |item, buf| {
                            buf.put_i32(*item);
                            Ok(())
                        },
                        buf,
                    )?;
                    Ok(())
                })
                .unwrap();
            }
            FrontendMessage::SASLInitialResponse { body, mechanism } => {
                buf.put_u8(b'p');
                write_message(&mut buf, |buf| {
                    write_cstr(buf, mechanism.as_bytes())?;
                    buf.put_i32(body.len() as i32);
                    buf.put_slice(body);
                    Ok(())
                })
                .unwrap();
            }
            FrontendMessage::SASLResponse { body } => {
                buf.put_u8(b'p');
                write_message(&mut buf, |buf| {
                    buf.extend_from_slice(body);
                    Ok(())
                })
                .unwrap();
            }
        }
        buf
    }

    pub async fn decode<T>(mut conn: T) -> Result<FrontendMessage, anyhow::Error>
    where
        T: AsyncRead + Unpin + AsyncReadExt + AsyncWrite + AsyncWriteExt,
    {
        let mut meta = [0; 1];
        conn.read_exact(&mut meta).await.map_err(|e| {
            error!("error while reading frontend meta [err_msg: {:?}]", e);
            anyhow!("invalid frontend message")
        })?;
        let len = decode_frame_length(&mut conn).await?;
        let mut buf = BytesMut::new();
        buf.resize(len, b'0');
        conn.read_exact(&mut buf).await?;
        match meta[0] {
            b'D' => match buf[0] {
                b'S' => {
                    buf.advance(1);
                    let name = read_cstr(&mut buf)?;
                    return Ok(FrontendMessage::Describe {
                        is_prepared_statement: true,
                        name: name,
                    });
                }
                b'P' => {
                    buf.advance(1);
                    let name = read_cstr(&mut buf)?;
                    return Ok(FrontendMessage::Describe {
                        is_prepared_statement: false,
                        name: name,
                    });
                }
                _ => {
                    return Err(anyhow!("invalid frontend message"));
                }
            },
            b'H' => return Ok(FrontendMessage::Flush),
            b'Q' => {
                let query_string = read_cstr(&mut buf)?;
                return Ok(FrontendMessage::Query { query_string });
            }
            b'S' => Ok(FrontendMessage::Sync),
            b'X' => {
                return Ok(FrontendMessage::Terminate);
            }
            b'B' => {
                let destination_portal_name = read_cstr(&mut buf)?;
                let prepared_statement_name = read_cstr(&mut buf)?;
                let parameter_format_codes = read_counted_message(&mut buf, |buf| {
                    let val = NetworkEndian::read_i16(&buf);
                    buf.advance(2);
                    Ok(val)
                })?;
                let parameter_values = read_counted_message(&mut buf, |buf| {
                    let len_of_data = NetworkEndian::read_i32(&buf);
                    buf.advance(4);
                    if len_of_data == -1 {
                        return Ok(Value::Null);
                    } else if len_of_data == 0 {
                        return Ok(Value::NotNull(Vec::new()));
                    }
                    let pos = buf.remaining() - buf.len();
                    let val = Value::NotNull(buf[pos..len_of_data as usize].to_vec());
                    buf.advance(len_of_data as usize);
                    Ok(val)
                })?;
                let result_column_format_codes = read_counted_message(&mut buf, |buf| {
                    let result = NetworkEndian::read_i16(&buf);
                    buf.advance(2);
                    Ok(result)
                })?;
                return Ok(FrontendMessage::Bind {
                    destination_portal_name,
                    prepared_statement_name,
                    parameter_format_codes,
                    parameter_values,
                    result_column_format_codes,
                });
            }
            b'C' => {
                let mut is_portal = false;
                if buf[0] != b'P' {
                    is_portal = true;
                }
                buf.advance(1);
                let name = read_cstr(&mut buf)?;
                Ok(FrontendMessage::Close { is_portal, name })
            }
            b'd' => Ok(FrontendMessage::CopyData(buf.to_vec())),
            b'c' => {
                return Ok(FrontendMessage::CopyDone);
            }
            b'f' => {
                let err_msg = read_cstr(&mut buf)?;
                Ok(FrontendMessage::CopyFail { err_msg })
            }
            b'E' => {
                let name = read_cstr(&mut buf)?;
                let max_no_of_rows = NetworkEndian::read_i32(&buf[0..]);
                return Ok(FrontendMessage::Execute {
                    name,
                    max_no_of_rows,
                });
            }
            b'F' => {
                let object_id = NetworkEndian::read_i32(&buf);
                buf.advance(4);
                let format_codes = read_counted_message(&mut buf, |buf| {
                    let format_code = NetworkEndian::read_i16(buf);
                    buf.advance(2);
                    Ok(format_code)
                })?;
                let function_arguments = read_counted_message(&mut buf, |buf| {
                    let arg_len = NetworkEndian::read_i32(buf);
                    buf.advance(4);
                    if arg_len == -1 {
                        return Ok(Value::Null);
                    } else if arg_len == 0 {
                        return Ok(Value::NotNull(Vec::new()));
                    }
                    let pos = buf.remaining() - buf.len();
                    let val = Value::NotNull(buf[pos..arg_len as usize].to_vec());
                    buf.advance(arg_len as usize);
                    Ok(val)
                })?;
                let result_format_code = NetworkEndian::read_i16(&buf);
                return Ok(FrontendMessage::FunctionCall {
                    object_id,
                    format_codes,
                    function_arguments,
                    result_format_code,
                });
            }
            b'P' => {
                let name = read_cstr(&mut buf)?;
                let query = read_cstr(&mut buf)?;
                let object_ids = read_counted_message(&mut buf, |buf| {
                    let format_code = NetworkEndian::read_i32(buf);
                    buf.advance(4);
                    Ok(format_code)
                })?;
                return Ok(FrontendMessage::Parse {
                    name,
                    query,
                    object_ids,
                });
            }
            _ => {
                return Err(anyhow!("unrecognized frontend message"));
            }
        }
    }
}
