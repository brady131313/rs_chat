use std::collections::HashMap;

use common::commands::Command;

use crate::{inputs::stateful_list::StatefulList, io::IoEvent};

use super::Message;

pub enum Active {
    Room,
    Chat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Rooms,
    Chats,
    Messages,
    NewMessage,
    Users,
    NewRoom,
    AllUsers,
    AllRooms,
}

impl Pane {
    pub fn title(&self) -> &'static str {
        match self {
            Pane::Rooms => "Active Rooms",
            Pane::Chats => "Private Chats",
            Pane::Messages => "Messages",
            Pane::NewMessage => "New Message",
            Pane::Users => "Room Users",
            Pane::NewRoom => "New Room",
            Pane::AllUsers => "All Users",
            Pane::AllRooms => "All Rooms",
        }
    }
}

pub struct State {
    pane: Pane,
    keep_alive: bool,
    pub new_room: String,
    pub new_message: String,
    pub active_rooms: StatefulList<String>,
    pub active_chats: StatefulList<String>,
    room_users: HashMap<String, StatefulList<String>>,
    room_messages: HashMap<String, StatefulList<Message>>,
    chat_messages: HashMap<String, StatefulList<Message>>,
    pub all_rooms: StatefulList<String>,
    pub all_users: StatefulList<String>,
}

impl State {
    pub fn focus_pane(&mut self, pane: Pane) {
        self.pane = pane
    }

    pub fn current_pane(&self) -> Pane {
        self.pane
    }

    pub fn set_keep_alive(&mut self, keep_alive: bool) {
        self.keep_alive = keep_alive
    }

    pub fn keep_alive(&self) -> bool {
        self.keep_alive
    }

    pub fn unselect_lists(&mut self) {
        self.current_room_users_mut()
            .map(|room| room.unselect())
            .unwrap_or_default();
        self.current_messages_mut()
            .map(|msg| msg.unselect())
            .unwrap_or_default();
        self.active_chats.unselect();
        self.active_rooms.unselect();
    }

    pub fn leave_room(&mut self) -> Option<IoEvent> {
        if let Some(room_idx) = self.active_rooms.selected() {
            let room = self.active_rooms.items[room_idx].to_owned();

            self.active_rooms.previous();
            self.active_rooms.items.remove(room_idx);
            self.room_users.remove(&room);
            self.room_messages.remove(&room);

            Some(IoEvent::Command(Command::Leave { room }))
        } else {
            None
        }
    }

    pub fn active_list(&self) -> Option<Active> {
        if self.active_rooms.selected_item().is_some() {
            Some(Active::Room)
        } else if self.active_chats.selected_item().is_some() {
            Some(Active::Chat)
        } else {
            None
        }
    }

    pub fn room_users_mut(&mut self, room: &str) -> Option<&mut StatefulList<String>> {
        self.room_users.get_mut(room)
    }

    pub fn current_room_users_mut(&mut self) -> Option<&mut StatefulList<String>> {
        if let Active::Room = self.active_list()? {
            let selected = self.active_rooms.selected_item()?;
            self.room_users.get_mut(selected)
        } else {
            None
        }
    }

    pub fn room_messages_mut(&mut self, room: &str) -> Option<&mut StatefulList<Message>> {
        self.room_messages.get_mut(room)
    }

    pub fn current_messages_mut(&mut self) -> Option<&mut StatefulList<Message>> {
        match self.active_list()? {
            Active::Room => {
                let selected = self.active_rooms.selected_item()?;
                self.room_messages.get_mut(selected)
            }
            Active::Chat => {
                let selected = self.active_chats.selected_item()?;
                self.chat_messages.get_mut(selected)
            }
        }
    }

    pub fn chat_messages_mut(&mut self, username: &str) -> Option<&mut StatefulList<Message>> {
        self.chat_messages.get_mut(username)
    }

    pub fn add_active_room(&mut self, room: String) {
        if !self.active_rooms.items.contains(&room) {
            self.active_rooms.items.push(room.clone());

            // Unselect a chat and select added room
            self.active_chats.unselect();
            self.active_rooms
                .state
                .select(Some(self.active_rooms.items.len() - 1));

            self.room_users
                .insert(room.clone(), StatefulList::with_items(vec![]));
            self.room_messages
                .insert(room, StatefulList::with_items(vec![]));
        }
    }

    pub fn add_chat(&mut self, username: String) {
        if !self.active_chats.items.contains(&username) {
            self.active_chats.items.push(username.clone());

            // Unselect room and select chat
            self.active_rooms.unselect();
            self.active_chats
                .state
                .select(Some(self.active_chats.items.len() - 1));

            self.chat_messages
                .insert(username, StatefulList::with_items(vec![]));
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            pane: Pane::Rooms,
            keep_alive: true,
            new_room: String::from(""),
            new_message: String::from(""),
            active_rooms: StatefulList::default(),
            active_chats: StatefulList::default(),
            room_users: HashMap::default(),
            room_messages: HashMap::default(),
            chat_messages: HashMap::default(),
            all_rooms: StatefulList::default(),
            all_users: StatefulList::default(),
        }
    }
}
