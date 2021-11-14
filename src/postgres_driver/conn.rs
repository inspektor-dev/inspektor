use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use std::task::{Poll};
use tokio_openssl::SslStream;

#[derive(Debug)]
pub enum PostgresConn {
    Unsecured(TcpStream),
    Secured(SslStream<TcpStream>),
}

impl AsyncRead for PostgresConn {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.get_mut() {
            PostgresConn::Unsecured(inner) => Pin::new(inner).poll_read(cx, buf),
            PostgresConn::Secured(inner) => Pin::new(inner).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for PostgresConn {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> Poll<std::result::Result<usize, std::io::Error>> {
        match self.get_mut() {
            PostgresConn::Unsecured(inner) => Pin::new(inner).poll_write(cx, buf),
            PostgresConn::Secured(inner) => Pin::new(inner).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            PostgresConn::Unsecured(inner) => Pin::new(inner).poll_flush(cx),
            PostgresConn::Secured(inner) => Pin::new(inner).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        match self.get_mut() {
            PostgresConn::Unsecured(inner) => Pin::new(inner).poll_shutdown(cx),
            PostgresConn::Secured(inner) => Pin::new(inner).poll_shutdown(cx),
        }
    }
}