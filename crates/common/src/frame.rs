use std::io::{self, Cursor, Read};

use thiserror::Error;
use tokio::io::AsyncWriteExt;

#[derive(Debug, PartialEq, Eq)]
pub struct Frame {
    raw: String,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum FrameError {
    #[error("stream ended early")]
    Incomplete,
}

impl Frame {
    pub fn parse(src: &mut Cursor<&[u8]>) -> Result<Self, FrameError> {
        let len = read_u32(src)?;

        let mut out = vec![0; len as usize];
        src.read_exact(&mut out)
            .map_err(|_| FrameError::Incomplete)?;

        let str = String::from_utf8(out).unwrap();
        Ok(Self { raw: str })
    }

    pub async fn write_stream<S: AsyncWriteExt + Unpin + Send>(
        &self,
        out: &mut S,
    ) -> io::Result<()> {
        out.write_u32(self.raw.len() as u32).await?;
        out.write_all(self.raw.as_bytes()).await?;
        out.flush().await
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }
}

fn read_u32(src: &mut Cursor<&[u8]>) -> Result<u32, FrameError> {
    let mut buf = [0; 4];
    src.read_exact(&mut buf)
        .map_err(|_| FrameError::Incomplete)?;

    let n = u32::from_be_bytes(buf);
    Ok(n)
}

impl<T: Into<String>> From<T> for Frame {
    fn from(t: T) -> Self {
        Self { raw: t.into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn round_trip_u32() {
        let u: u32 = u32::MAX;
        let mut out: Vec<u8> = Vec::new();

        out.write_u32(u).await.unwrap();

        let mut cursor = Cursor::new(&out[..]);
        let res = read_u32(&mut cursor).unwrap();

        assert_eq!(u, res);
    }

    #[tokio::test]
    async fn incomplete_u32() {
        let mut out: Vec<u8> = Vec::new();
        out.write_u16(u16::MAX).await.unwrap();

        let mut cursor = Cursor::new(&out[..]);
        assert_eq!(read_u32(&mut cursor).unwrap_err(), FrameError::Incomplete);
    }

    #[tokio::test]
    async fn write_frame() {
        let frame =
            Frame::from("some really long string that has to get encoded with utf8 Здравствуйте");
        let mut out = Vec::new();
        frame.write_stream(&mut out).await.unwrap();

        let mut cursor = Cursor::new(&out[..]);
        let parsed = Frame::parse(&mut cursor).unwrap();
        assert_eq!(frame, parsed);
    }
}
