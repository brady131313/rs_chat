use std::collections::HashMap;

use common::commands::Command;
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    inputs::{key::Key, stateful_list::StatefulList},
    io::IoEvent,
};

macro_rules! key {
    (up) => {
        Key::Char('k') | Key::Up
    };

    (down) => {
        Key::Char('j') | Key::Down
    };
}

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Rooms,
    Messages,
    Users,
    NewRoom,
    AllUsers,
    AllRooms,
}

impl Pane {
    pub fn title(&self) -> &'static str {
        match self {
            Pane::Rooms => "Active Rooms",
            Pane::Messages => "Messages",
            Pane::Users => "Room Users",
            Pane::NewRoom => "New Room",
            Pane::AllUsers => "All Users",
            Pane::AllRooms => "All Rooms",
        }
    }
}

pub struct App {
    pane: Pane,
    io_tx: UnboundedSender<IoEvent>,
    new_room: String,
    pub active_rooms: StatefulList<String>,
    room_users: HashMap<String, StatefulList<String>>,
    pub all_rooms: StatefulList<String>,
    pub all_users: StatefulList<String>,
}

impl App {
    pub fn new(io_tx: UnboundedSender<IoEvent>) -> Self {
        Self {
            io_tx,
            pane: Pane::Rooms,
            new_room: String::from(""),
            active_rooms: StatefulList::with_items(Vec::new()),
            room_users: HashMap::new(),
            all_rooms: StatefulList::with_items(Vec::new()),
            all_users: StatefulList::with_items(Vec::new()),
        }
    }

    pub fn current_pane(&self) -> Pane {
        self.pane
    }

    pub fn new_room(&self) -> &str {
        &self.new_room
    }

    pub fn leave_room(&mut self) {
        if let Some(room_idx) = self.active_rooms.selected() {
            let room = self.active_rooms.items[room_idx].to_owned();

            self.active_rooms.previous();
            self.active_rooms.items.remove(room_idx);
            self.room_users.remove(&room);

            self.dispatch(IoEvent::Command(Command::Leave { room }))
        }
    }

    pub fn room_users_mut(&mut self, room: &str) -> Option<&mut StatefulList<String>> {
        self.room_users.get_mut(room)
    }

    pub fn current_room_users_mut(&mut self) -> Option<&mut StatefulList<String>> {
        let selected = self.active_rooms.selected_item()?;
        self.room_users.get_mut(selected)
    }

    pub fn current_room_users(&self) -> Option<&Vec<String>> {
        let selected = self.active_rooms.selected_item()?;
        self.room_users.get(selected).map(|l| &l.items)
    }

    pub fn add_active_room(&mut self, room: String) {
        if !self.active_rooms.items.contains(&room) {
            self.active_rooms.items.push(room.clone());
            self.room_users
                .insert(room, StatefulList::with_items(vec![]));
        }
    }

    pub fn dispatch(&mut self, event: IoEvent) {
        if let Err(e) = self.io_tx.send(event) {
            panic!("Error dispatching {e}")
        }
    }

    pub fn update_on_tick(&mut self) -> AppReturn {
        AppReturn::Continue
    }

    pub fn do_action(&mut self, key: Key) -> AppReturn {
        if matches!(key, Key::Ctrl('c') | Key::Char('q')) {
            return AppReturn::Exit;
        }

        if key == Key::Esc {
            self.pane = Pane::Rooms;
            return AppReturn::Continue;
        }

        if key == Key::Ctrl('s') {
            self.dispatch(IoEvent::Sleep);
            return AppReturn::Continue;
        }

        match self.pane {
            Pane::Rooms => self.room_action(key),
            Pane::Messages => todo!(),
            Pane::Users => self.users_action(key),
            Pane::NewRoom => self.new_room_action(key),
            Pane::AllUsers => self.all_users_action(key),
            Pane::AllRooms => self.all_rooms_action(key),
        }
    }

    fn room_action(&mut self, key: Key) -> AppReturn {
        match key {
            Key::Char('a') => {
                self.pane = Pane::NewRoom;
                AppReturn::Continue
            }
            Key::Char('l') => {
                self.leave_room();
                AppReturn::Continue
            }
            Key::Char('u') => {
                self.pane = Pane::Users;
                AppReturn::Continue
            }
            Key::Char('i') => {
                self.pane = Pane::Messages;
                AppReturn::Continue
            }
            Key::Char('U') => {
                self.dispatch(IoEvent::Command(Command::ListUsers));
                self.pane = Pane::AllUsers;
                AppReturn::Continue
            }
            Key::Char('R') => {
                self.dispatch(IoEvent::Command(Command::ListRooms));
                self.pane = Pane::AllRooms;
                AppReturn::Continue
            }
            key!(up) => {
                self.active_rooms.previous();
                AppReturn::Continue
            }
            key!(down) => {
                self.active_rooms.next();
                AppReturn::Continue
            }
            _ => AppReturn::Continue,
        }
    }

    fn users_action(&mut self, key: Key) -> AppReturn {
        match key {
            key!(up) => {
                self.current_room_users_mut()
                    .map(|room| room.previous())
                    .unwrap_or_default();
                AppReturn::Continue
            }
            key!(down) => {
                self.current_room_users_mut()
                    .map(|room| room.next())
                    .unwrap_or_default();
                AppReturn::Continue
            }
            _ => AppReturn::Continue,
        }
    }

    fn all_users_action(&mut self, key: Key) -> AppReturn {
        match key {
            key!(up) => {
                self.all_users.previous();
                AppReturn::Continue
            }
            key!(down) => {
                self.all_users.next();
                AppReturn::Continue
            }
            _ => AppReturn::Continue,
        }
    }

    fn all_rooms_action(&mut self, key: Key) -> AppReturn {
        match key {
            key!(up) => {
                self.all_rooms.previous();
                AppReturn::Continue
            }
            key!(down) => {
                self.all_rooms.next();
                AppReturn::Continue
            }
            Key::Enter => {
                if let Some(room) = self.all_rooms.selected_item() {
                    self.dispatch(IoEvent::Command(Command::JoinOrCreate {
                        room: room.clone(),
                    }));
                    self.pane = Pane::Rooms;
                }
                AppReturn::Continue
            }
            _ => AppReturn::Continue,
        }
    }

    fn new_room_action(&mut self, key: Key) -> AppReturn {
        match key {
            Key::Enter => {
                self.dispatch(IoEvent::Command(Command::JoinOrCreate {
                    room: self.new_room.clone(),
                }));
                self.new_room.clear();
                self.pane = Pane::Rooms;
                AppReturn::Continue
            }
            Key::Backspace => {
                self.new_room.pop();
                AppReturn::Continue
            }
            Key::Char(c) => {
                self.new_room.push(c);
                AppReturn::Continue
            }
            _ => AppReturn::Continue,
        }
    }
}
