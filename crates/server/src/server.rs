use std::io;

use common::{
    commands::{Command, Response},
    connection::Connection,
    frame::Frame,
    Result,
};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    sync::mpsc,
};
use tracing::instrument;

use crate::state::{Peer, ResponseType, ServerState};

pub struct Server {
    listener: TcpListener,
}

pub struct Handler {
    connection: Connection,
    state: ServerState,
    rx: mpsc::UnboundedReceiver<Response>,
}

impl Server {
    pub async fn bind(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        Ok(Self { listener })
    }

    pub async fn listen(&self) -> io::Result<()> {
        tracing::info!("accepting connections at {}", self.listener.local_addr()?);

        let state = ServerState::default();
        loop {
            let (socket, addr) = self.listener.accept().await?;
            tracing::info!("received connection from {addr}");

            let (tx, rx) = mpsc::unbounded_channel();
            let peer = Peer { addr, tx };

            let mut handler = Handler {
                connection: Connection::new(socket),
                state: state.clone(),
                rx,
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run(peer).await {
                    tracing::error!(%err);
                }
            });
        }
    }
}

impl Handler {
    #[instrument(level = "info", name = "Handler::run", skip(self, peer), fields(peer_addr = %peer.addr))]
    async fn run(&mut self, peer: Peer) -> Result<()> {
        loop {
            tokio::select! {
                Some(res) = self.rx.recv() => {
                    tracing::trace!("broadcast {res:?}");
                    let frame = Frame::from(String::from(res));
                    self.connection.write_frame(&frame).await?;
                }
                frame = self.connection.read_frame() => {
                    let frame = match frame? {
                        Some(frame) => frame,
                        None => break
                    };

                    let command: Command = frame.raw().try_into()?;
                    tracing::trace!("{command:?}");
                    let response = self.state.apply(command, peer.clone());

                    match response {
                        ResponseType::None => {},
                        ResponseType::Sender(cmd) => {
                            let frame = Frame::from(String::from(cmd));
                            self.connection.write_frame(&frame).await?;
                        },
                        ResponseType::Broadcast(res) => self.state.broadcast(res),
                        ResponseType::BroadcastRoom(room, res) => self.state.broadcast_room(&room, res)
                    }
                }
            }
        }
        Ok(())
    }
}
