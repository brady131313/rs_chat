use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use common::commands::{Command, Response, ResponseError, Target};
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
    pub addr: SocketAddr,
    pub tx: mpsc::UnboundedSender<Response>,
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
            Target::Username(..) => todo!(),
        }
    }

    fn user(&self, addr: SocketAddr) -> &str {
        self.addr_to_user.get(&addr).unwrap()
    }
}

impl ServerState {
    pub fn apply(&self, command: Command, peer: Peer) -> ResponseType {
        let mut state = self.shared.state.lock().unwrap();
        match command {
            Command::Hello { username } => state.hello(username, peer),
            Command::JoinOrCreate { room } => state.join_or_create(room, peer.addr),
            Command::Leave { room } => state.leave_room(room, peer.addr),
            Command::KeepAlive => todo!(),
            Command::ListRooms => state.list_rooms(),
            Command::ListUsers => state.list_users(),
            Command::Send { target, message } => state.send(target, message, peer.addr),
        }
    }

    pub fn broadcast(&self, response: Response) {
        for user in self.shared.state.lock().unwrap().users.values() {
            user.tx.send(response.clone()).unwrap();
        }
    }

    pub fn broadcast_room(&self, room: &str, response: Response) {
        let state = self.shared.state.lock().unwrap();
        for user in &state.rooms[room] {
            let user = &state.users[user];
            user.tx.send(response.clone()).unwrap()
        }
    }
}
