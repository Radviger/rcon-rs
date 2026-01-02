use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use std::io::prelude::*;
use std::io::Error as IoError;
use std::fmt::Display;
use std::string::FromUtf8Error;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use thiserror::Error;

pub struct RCon {
    stream: TcpStream,
    authorized: bool,
    request_id: i32
}

impl RCon {
    pub fn connect(socket: &SocketAddr, timeout: Duration) -> Result<RCon, RConError> {
        let stream = TcpStream::connect_timeout(socket, timeout)?;
        stream.set_read_timeout(Some(timeout))?;
        stream.set_write_timeout(Some(timeout))?;

        Ok(RCon {
            stream, authorized: false, request_id: 0
        })
    }

    fn send(&mut self, ty: i32, msg: &str) -> Result<(), RConError> {
        let raw_msg = msg.as_bytes();
        let mut buf = Vec::new();
        buf.write_i32::<LE>(4 + 4 + raw_msg.len() as i32 + 2)?;
        let request_id = self.next_request_id();
        buf.write_i32::<LE>(request_id)?;
        buf.write_i32::<LE>(ty)?;
        buf.write_all(&raw_msg)?;
        buf.write_all(&[0x00, 0x00])?;
        self.stream.write_all(&buf)?;
        self.stream.flush()?;
        Ok(())
    }

    fn next_request_id(&mut self) -> i32 {
        self.request_id += 1;
        self.request_id
    }

    pub fn is_authorized(&self) -> bool {
        self.authorized
    }

    fn read_response(&mut self) -> Result<String, RConError> {
        let mut response = Vec::new();

        while let Ok(len) = self.stream.read_i32::<LE>() {
            if len >= 10 {
                let request_id = self.stream.read_i32::<LE>()?;
                let ty = self.stream.read_i32::<LE>()?;
                let mut buf = [0u8; 8192];
                if request_id == -1 {
                    return Err(RConError::NotAuthorized);
                }
                let cnt = len as usize - 10;
                if cnt > 0 {
                    let slice = &mut buf[0..cnt];
                    self.stream.read_exact(slice)?;
                    response.extend_from_slice(slice);
                }
                let _ = self.stream.read_u16::<LE>()?;
                if cnt < 8192 {
                    break;
                }
            } else {
                return Err(RConError::LengthTooSmall(len as u32))
            }
        }

        Ok(String::from_utf8(response)?)
    }

    pub fn authorize(&mut self, password: &str) -> Result<bool, RConError> {
        self.send(3, password)?;
        self.read_response().map(|r| r.is_empty())
    }

    pub fn command(&mut self, command: &str) -> Result<String, RConError> {
        self.send(2, command)?;
        self.read_response()
    }
}

#[derive(Error, Debug)]
pub enum RConError {
    #[error("{0}")]
    Io(#[from] IoError),
    #[error("{0}")]
    Utf8(#[from] FromUtf8Error),
    #[error("Not authorized!")]
    NotAuthorized,
    #[error("Packet length too small: {0}")]
    LengthTooSmall(u32),
    #[error("Packet length mismatch! Expected: {0}, got : {1}")]
    LengthMismatch(usize, usize)
}