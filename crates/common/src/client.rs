use tokio::net::{TcpStream, ToSocketAddrs};

use crate::{
    commands::{Command, Response},
    connection::Connection,
    frame::Frame,
    Error, Result,
};

pub struct Client {
    connection: Connection,
    username: String,
}

impl Client {
    pub async fn connect(addr: impl ToSocketAddrs, username: String) -> Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        println!("Client connected to server at {}", stream.peer_addr()?);

        let connection = Connection::new(stream);
        Ok(Self {
            connection,
            username,
        })
    }

    pub async fn hello(&mut self) -> Result<()> {
        self.write_command(Command::Hello {
            username: self.username.clone(),
        })
        .await
    }

    pub async fn write_command(&mut self, command: Command) -> Result<()> {
        let frame = Frame::from(String::from(command));
        self.connection.write_frame(&frame).await?;
        Ok(())
    }

    pub async fn read_response(&mut self) -> Result<Response> {
        let frame = self.connection.read_frame().await?;

        match frame {
            Some(frame) => {
                let response: Response = frame.raw().try_into()?;
                Ok(response)
            }
            None => Err(Error::ConnectionResetByPeer),
        }
    }
}
