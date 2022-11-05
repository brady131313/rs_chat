use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use common::commands::{Command, Kill, Response, ResponseError, Target};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Default)]
pub struct ServerState {
    shared: Arc<Shared>,
}

#[derive(Debug, Default)]
struct Shared {
    state: Mutex<State>,
}

#[derive(Debug, Clone)]
pub struct Peer {
    addr: SocketAddr,
    tx: mpsc::UnboundedSender<Response>,
    keep_alive: bool,
    kill_tx: mpsc::UnboundedSender<Kill>,
}

impl Peer {
    pub fn new(
        addr: SocketAddr,
        tx: mpsc::UnboundedSender<Response>,
        kill_tx: mpsc::UnboundedSender<Kill>,
    ) -> Self {
        Self {
            addr,
            tx,
            keep_alive: true,
            kill_tx,
        }
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }
}

#[derive(Debug, Default)]
struct State {
    addr_to_user: HashMap<SocketAddr, String>,
    users: HashMap<String, Peer>,
    rooms: HashMap<String, HashSet<String>>,
}

pub enum ResponseType {
    None,
    Sender(Response),
    SenderAndUser(String, Response),
    Broadcast(Response),
    BroadcastRoom(String, Response),
}

impl State {
    fn hello(&mut self, username: String, peer: Peer) -> ResponseType {
        let addr = peer.addr;
        if self.users.insert(username.clone(), peer).is_some() {
            return ResponseType::Sender(Response::Err(ResponseError::UserAlreadyExists(username)));
        }

        self.addr_to_user.insert(addr, username);
        ResponseType::None
    }

    fn join_or_create(&mut self, room: String, user: SocketAddr) -> ResponseType {
        let user = self.user(user).into();

        let room_entry = self.rooms.entry(room.clone()).or_default();
        room_entry.insert(user);

        let users = room_entry.iter().cloned().collect();
        let response = Response::ListMembers {
            room: room.clone(),
            users,
        };

        ResponseType::BroadcastRoom(room, response)
    }

    fn leave_room(&mut self, room: String, user: SocketAddr) -> ResponseType {
        let user = self.user(user).to_owned();

        if let Some(room_entry) = self.rooms.get_mut(&room) {
            if room_entry.remove(&user) {
                let users = room_entry.iter().cloned().collect();
                let response = Response::ListMembers {
                    room: room.clone(),
                    users,
                };

                ResponseType::BroadcastRoom(room, response)
            } else {
                ResponseType::Sender(Response::Err(ResponseError::UserNotInRoom { user, room }))
            }
        } else {
            ResponseType::Sender(Response::Err(ResponseError::RoomDoesNotExist(room)))
        }
    }

    fn list_rooms(&self) -> ResponseType {
        let rooms = self.rooms.keys().cloned().collect();
        ResponseType::Sender(Response::ListRooms { rooms })
    }

    fn list_users(&self) -> ResponseType {
        let users = self.users.keys().cloned().collect();
        ResponseType::Sender(Response::ListUsers { users })
    }

    fn send(&mut self, target: Target, message: String, user: SocketAddr) -> ResponseType {
        let user = self.user(user).to_owned();

        match target {
            Target::Room(room) => {
                let response = Response::TellRoom {
                    room: room.clone(),
                    sender: user,
                    message,
                };
                ResponseType::BroadcastRoom(room, response)
            }
            Target::Username(username) => {
                let response = Response::TellUser {
                    username: username.clone(),
                    sender: user,
                    message,
                };
                ResponseType::SenderAndUser(username, response)
            }
        }
    }

    fn keep_alive(&mut self, user: SocketAddr) -> ResponseType {
        let user = self.user(user).to_owned();
        let user = self.users.get_mut(&user).unwrap();
        user.keep_alive = true;
        ResponseType::None
    }

    fn user(&self, addr: SocketAddr) -> &str {
        self.addr_to_user.get(&addr).unwrap()
    }

    fn kick_keep_alive(&mut self) {
        for peer in self.users.values_mut() {
            if !peer.keep_alive {
                peer.kill_tx.send(Kill).unwrap();
            } else {
                peer.keep_alive = false;
            }
        }
    }

    fn users_rooms_mut<'a>(
        &'a mut self,
        user: &'a str,
    ) -> impl Iterator<Item = (&String, &mut HashSet<String>)> {
        self.rooms
            .iter_mut()
            .filter(|(_, users)| users.contains(user))
    }

    fn remove_peer(&mut self, peer: &Peer) {
        let user = self.user(peer.addr).to_owned();
        self.addr_to_user.remove(&peer.addr);
        self.users.remove(&user);

        // Remove user from each room they're in and get a list of updated users
        // to send to all users in the room
        let mut rooms_to_notify = Vec::new();
        for (room, users) in self.users_rooms_mut(&user) {
            users.remove(&user);
            rooms_to_notify.push((
                room.clone(),
                Response::ListMembers {
                    room: room.clone(),
                    users: users.iter().cloned().collect(),
                },
            ));
        }

        for (room, response) in rooms_to_notify {
            self.broadcast_room(&room, response)
        }
    }

    fn broadcast_room(&self, room: &str, response: Response) {
        for user in &self.rooms[room] {
            let user = &self.users[user];
            user.tx.send(response.clone()).unwrap()
        }
    }
}

impl ServerState {
    pub fn apply(&self, command: Command, peer: Peer) -> ResponseType {
        let mut state = self.shared.state.lock().unwrap();
        match command {
            Command::Hello { username } => state.hello(username, peer),
            Command::JoinOrCreate { room } => state.join_or_create(room, peer.addr),
            Command::Leave { room } => state.leave_room(room, peer.addr),
            Command::KeepAlive => state.keep_alive(peer.addr),
            Command::ListRooms => state.list_rooms(),
            Command::ListUsers => state.list_users(),
            Command::Send { target, message } => state.send(target, message, peer.addr),
        }
    }

    pub fn send(&self, user: &str, response: Response) {
        let state = self.shared.state.lock().unwrap();
        if let Some(peer) = state.users.get(user) {
            peer.tx.send(response).unwrap();
        }
    }

    pub fn broadcast(&self, response: Response) {
        for user in self.shared.state.lock().unwrap().users.values() {
            user.tx.send(response.clone()).unwrap();
        }
    }

    pub fn broadcast_room(&self, room: &str, response: Response) {
        let state = self.shared.state.lock().unwrap();
        state.broadcast_room(room, response)
    }

    pub fn kick_keep_alive(&self) {
        let mut state = self.shared.state.lock().unwrap();
        state.kick_keep_alive();
    }

    pub fn remove_peer(&self, peer: &Peer) {
        let mut state = self.shared.state.lock().unwrap();
        state.remove_peer(peer)
    }
}
