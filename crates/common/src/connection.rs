use std::io::{self, Cursor};

use bytes::{Buf, BytesMut};
use tokio::{
    io::{AsyncReadExt, BufWriter},
    net::TcpStream,
};

use crate::{
    frame::{Frame, FrameError},
    Error, Result,
};

const READ_BUFFER_CAPACITY: usize = 16 * 1024;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    // Buffer for reading frames
    buffer: BytesMut,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(READ_BUFFER_CAPACITY),
        }
    }

    /// Read a single frame from stream
    ///
    /// waits until enough data is retrieved to parse frame and any
    /// remaining data is kept there until next call
    ///
    /// If stream is closed in way that doesn't break frame return none,
    /// otherwise error
    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if 0 == self.stream.read_buf(&mut self.buffer).await? {
                // clean shutdown if empty buffer, otherwise peer closed while sending frame
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(Error::ConnectionResetByPeer);
                }
            }
        }
    }

    fn parse_frame(&mut self) -> Result<Option<Frame>> {
        let mut buf = Cursor::new(&self.buffer[..]);

        match Frame::parse(&mut buf) {
            Ok(frame) => {
                self.buffer.advance(4 + frame.raw().len());
                Ok(Some(frame))
            }
            Err(FrameError::Incomplete) => Ok(None),
            Err(err) => Err(err.into()),
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
        frame.write_stream(&mut self.stream).await
    }
}
