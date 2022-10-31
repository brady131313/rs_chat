use std::sync::mpsc::Sender;

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
    io_tx: Sender<IoEvent>,
    pub active_rooms: StatefulList<String>,
    pub room_users: StatefulList<String>,
    pub all_rooms: StatefulList<String>,
    pub all_users: StatefulList<String>,
}

impl App {
    pub fn new(io_tx: Sender<IoEvent>) -> Self {
        Self {
            io_tx,
            pane: Pane::Rooms,
            active_rooms: StatefulList::with_items(Vec::new()),
            room_users: StatefulList::with_items(Vec::new()),
            all_rooms: StatefulList::with_items(Vec::new()),
            all_users: StatefulList::with_items(Vec::new()),
        }
    }

    pub fn current_pane(&self) -> Pane {
        self.pane
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

        if key == Key::Char('s') {
            self.dispatch(IoEvent::Sleep);
            return AppReturn::Continue;
        }

        match self.pane {
            Pane::Rooms => self.room_action(key),
            Pane::Messages => todo!(),
            Pane::Users => self.users_action(key),
            Pane::NewRoom => todo!(),
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
            Key::Char('u') => {
                self.pane = Pane::Users;
                AppReturn::Continue
            }
            Key::Char('i') => {
                self.pane = Pane::Messages;
                AppReturn::Continue
            }
            Key::Char('U') => {
                self.pane = Pane::AllUsers;
                AppReturn::Continue
            }
            Key::Char('R') => {
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
                self.room_users.previous();
                AppReturn::Continue
            }
            key!(down) => {
                self.room_users.next();
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
            _ => AppReturn::Continue,
        }
    }
}
