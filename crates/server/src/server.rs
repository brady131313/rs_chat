use std::{io, time::Duration};

use common::{
    commands::{Command, KeepAlive, Kill, Response, KEEP_ALIVE_CHECK, KEEP_ALIVE_INTERVAL},
    connection::Connection,
    frame::Frame,
    Result,
};
use tokio::{
    net::{TcpListener, ToSocketAddrs},
    sync::{mpsc, watch},
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
    keep_alive_rx: watch::Receiver<KeepAlive>,
    keep_alive_kill_rx: mpsc::UnboundedReceiver<Kill>,
}

impl Server {
    pub async fn bind(addr: impl ToSocketAddrs) -> io::Result<Self> {
        let listener = TcpListener::bind(addr).await?;

        Ok(Self { listener })
    }

    pub async fn listen(&self) -> io::Result<()> {
        tracing::info!("accepting connections at {}", self.listener.local_addr()?);

        let state = ServerState::default();

        // Broadcast keep alive
        let (keep_alive_tx, keep_alive_rx) = watch::channel(KeepAlive);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(KEEP_ALIVE_INTERVAL)).await;
                keep_alive_tx.send(KeepAlive).unwrap();
            }
        });

        // Kick clients that haven't kept alive
        let keep_alive_state = state.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(KEEP_ALIVE_CHECK)).await;
                keep_alive_state.kick_keep_alive();
            }
        });

        // handle incoming connections
        loop {
            let (socket, addr) = self.listener.accept().await?;
            tracing::info!("received connection from {addr}");

            let (tx, rx) = mpsc::unbounded_channel();
            let (keep_alive_kill_tx, keep_alive_kill_rx) = mpsc::unbounded_channel();
            let peer = Peer::new(addr, tx, keep_alive_kill_tx);

            let mut handler = Handler {
                connection: Connection::new(socket),
                state: state.clone(),
                rx,
                keep_alive_rx: keep_alive_rx.clone(),
                keep_alive_kill_rx,
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
    #[instrument(level = "info", name = "Handler::run", skip(self, peer), fields(peer_addr = %peer.addr()))]
    async fn run(&mut self, peer: Peer) -> Result<()> {
        loop {
            tokio::select! {
                _ = self.keep_alive_kill_rx.recv() => {
                    tracing::info!("killing");
                    self.state.remove_peer(&peer);
                    break;
                }
                _ = self.keep_alive_rx.changed() => {
                    self.state.broadcast(Response::KeepAlive)
                }
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
                        ResponseType::SenderAndUser(user, cmd) => {
                            self.state.send(&user, cmd.clone());
                            let frame = Frame::from(String::from(cmd));
                            self.connection.write_frame(&frame).await?;
                        }
                        ResponseType::Broadcast(res) => self.state.broadcast(res),
                        ResponseType::BroadcastRoom(room, res) => self.state.broadcast_room(&room, res)
                    }
                }
            }
        }
        Ok(())
    }
}
