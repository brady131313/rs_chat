use common::commands::{Command, Target};
use tokio::sync::mpsc::UnboundedSender;

use crate::{inputs::key::Key, io::IoEvent};

use self::{
    actions::{Action, Actions},
    state::{Active, Pane, State},
};

pub mod actions;
pub mod state;

pub type Message = (String, String);

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub struct App {
    actions: Actions,
    io_tx: UnboundedSender<IoEvent>,
    pub state: State,
}

impl App {
    pub fn new(io_tx: UnboundedSender<IoEvent>) -> Self {
        let mut app = Self {
            io_tx,
            actions: Actions::from(vec![Action::Quit]),
            state: State::default(),
        };

        app.focus_pane(Pane::Rooms);
        app
    }

    pub fn dispatch(&mut self, event: IoEvent) {
        if let Err(_e) = self.io_tx.send(event) {
            // panic!("Error dispatching {e}")
        }
    }

    pub fn update_on_tick(&mut self) -> AppReturn {
        AppReturn::Continue
    }

    pub fn current_actions(&self) -> &Actions {
        &self.actions
    }

    fn focus_pane(&mut self, pane: Pane) {
        self.state.focus_pane(pane);

        let actions = match pane {
            state::Pane::Rooms => vec![
                Action::NewRoom,
                Action::LeaveRoom,
                Action::RoomUsers,
                Action::Messages,
                Action::MaybeFocusNewMessage,
                Action::Chats,
                Action::AllUsers,
                Action::AllRooms,
                Action::ListPrev,
                Action::ListNext,
                Action::Quit,
                Action::Sleep,
            ],
            state::Pane::Chats => vec![
                Action::Messages,
                Action::MaybeFocusNewMessage,
                Action::AllUsers,
                Action::AllRooms,
                Action::ListPrev,
                Action::ListNext,
                Action::Escape,
                Action::Quit,
                Action::Sleep,
            ],
            state::Pane::Messages => vec![
                Action::FocusNewMessage,
                Action::ListPrev,
                Action::ListNext,
                Action::Escape,
                Action::Quit,
                Action::Sleep,
            ],
            state::Pane::NewMessage => vec![Action::SendMessage, Action::Escape],
            state::Pane::Users => vec![
                Action::NewChat,
                Action::ListPrev,
                Action::ListNext,
                Action::AllUsers,
                Action::AllRooms,
                Action::Escape,
                Action::Quit,
                Action::Sleep,
            ],
            state::Pane::NewRoom => vec![Action::JoinOrCreateRoom, Action::Escape],
            state::Pane::AllUsers => vec![
                Action::NewChat,
                Action::ListPrev,
                Action::ListNext,
                Action::AllUsers,
                Action::AllRooms,
                Action::Escape,
                Action::Quit,
                Action::Sleep,
            ],
            state::Pane::AllRooms => vec![
                Action::JoinRoom,
                Action::ListPrev,
                Action::ListNext,
                Action::AllUsers,
                Action::AllRooms,
                Action::Escape,
                Action::Quit,
                Action::Sleep,
            ],
        };

        self.actions = Actions::from(actions);
    }

    pub fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            match action {
                Action::Quit => AppReturn::Exit,
                Action::Sleep => {
                    self.dispatch(IoEvent::Sleep);
                    AppReturn::Continue
                }
                Action::NewRoom => {
                    self.focus_pane(Pane::NewRoom);
                    AppReturn::Continue
                }
                Action::LeaveRoom => {
                    if let Some(event) = self.state.leave_room() {
                        self.dispatch(event);
                    }
                    AppReturn::Continue
                }
                Action::RoomUsers => {
                    self.focus_pane(Pane::Users);
                    AppReturn::Continue
                }
                Action::Messages => {
                    self.focus_pane(Pane::Messages);
                    AppReturn::Continue
                }
                Action::MaybeFocusNewMessage | Action::FocusNewMessage => {
                    if self.state.active_list().is_some() {
                        self.focus_pane(Pane::NewMessage);
                    }
                    AppReturn::Continue
                },
                Action::Chats => {
                    self.state.active_rooms.unselect();
                    self.focus_pane(Pane::Chats);
                    AppReturn::Continue
                }
                Action::AllUsers => {
                    self.dispatch(IoEvent::Command(Command::ListUsers));
                    self.focus_pane(Pane::AllUsers);
                    AppReturn::Continue
                }
                Action::AllRooms => {
                    self.dispatch(IoEvent::Command(Command::ListRooms));
                    self.focus_pane(Pane::AllRooms);
                    AppReturn::Continue
                }
                Action::ListPrev => {
                    match self.state.current_pane() {
                        Pane::Rooms => self.state.active_rooms.previous(),
                        Pane::Chats => self.state.active_chats.previous(),
                        Pane::Messages => self
                            .state
                            .current_messages_mut()
                            .map(|l| l.previous())
                            .unwrap_or_default(),
                        Pane::Users => self
                            .state
                            .current_room_users_mut()
                            .map(|l| l.previous())
                            .unwrap_or_default(),
                        Pane::AllUsers => self.state.all_users.previous(),
                        Pane::AllRooms => self.state.all_rooms.previous(),
                        Pane::NewRoom | Pane::NewMessage => unreachable!(),
                    };
                    AppReturn::Continue
                }
                Action::ListNext => {
                    match self.state.current_pane() {
                        Pane::Rooms => self.state.active_rooms.next(),
                        Pane::Chats => self.state.active_chats.next(),
                        Pane::Messages => self
                            .state
                            .current_messages_mut()
                            .map(|l| l.next())
                            .unwrap_or_default(),
                        Pane::Users => self
                            .state
                            .current_room_users_mut()
                            .map(|l| l.next())
                            .unwrap_or_default(),
                        Pane::AllUsers => self.state.all_users.next(),
                        Pane::AllRooms => self.state.all_rooms.next(),
                        Pane::NewRoom | Pane::NewMessage => unreachable!(),
                    };
                    AppReturn::Continue
                }
                Action::NewChat => {
                    if let Some(user) = self.state.all_users.selected_item().cloned() {
                        self.state.add_chat(user);
                        self.focus_pane(Pane::Chats)
                    }
                    AppReturn::Continue
                }
                Action::JoinRoom => {
                    if let Some(room) = self.state.all_rooms.selected_item().cloned() {
                        self.dispatch(IoEvent::Command(Command::JoinOrCreate { room }));
                        self.focus_pane(Pane::Rooms)
                    }
                    AppReturn::Continue
                }
                Action::JoinOrCreateRoom => {
                    self.dispatch(IoEvent::Command(Command::JoinOrCreate {
                        room: self.state.new_room.to_owned(),
                    }));
                    self.state.new_room.clear();
                    self.focus_pane(Pane::Rooms);
                    AppReturn::Continue
                }
                Action::SendMessage => {
                    let target = match self.state.active_list().unwrap() {
                        Active::Room => {
                            let room = self.state.active_rooms.selected_item().unwrap();
                            Target::Room(room.clone())
                        }
                        Active::Chat => {
                            let user = self.state.active_chats.selected_item().unwrap();
                            Target::Username(user.clone())
                        }
                    };

                    self.dispatch(IoEvent::Command(Command::Send {
                        target,
                        message: self.state.new_message.to_owned(),
                    }));

                    self.state.new_message.clear();
                    AppReturn::Continue
                }
                Action::Escape => {
                    self.focus_pane(Pane::Rooms);
                    self.state.unselect_lists();
                    AppReturn::Continue
                }
            }
        } else {
            if matches!(self.state.current_pane(), Pane::NewRoom | Pane::NewMessage) {
                let input = match self.state.current_pane() {
                    Pane::NewMessage => &mut self.state.new_message,
                    Pane::NewRoom => &mut self.state.new_room,
                    _ => unreachable!(),
                };

                match key {
                    Key::Backspace => {
                        input.pop();
                    }
                    Key::Char(c) => {
                        input.push(c);
                    }
                    _ => {}
                };
            }
            AppReturn::Continue
        }
    }
}
