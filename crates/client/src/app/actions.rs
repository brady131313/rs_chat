use std::fmt::Display;

use crate::inputs::key::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Quit the application
    Quit,
    /// Demo sleep to show async thread
    Sleep,
    /// Open modal to create/join a room
    NewRoom,
    /// Leave the selected room
    LeaveRoom,
    /// Focus users pane for selected room
    RoomUsers,
    /// Focus message pane
    Messages,
    /// Focus new message for selected room/user
    MaybeFocusNewMessage,
    /// Focus new message
    FocusNewMessage,
    /// Focus private chats
    Chats,
    /// Open modal of all users
    AllUsers,
    /// Open modal of all rooms
    AllRooms,
    /// Select prev item of active list
    ListPrev,
    /// Select next item of active list
    ListNext,
    /// Start new chat with selected user of all users modal
    NewChat,
    /// Join room of selected room from all rooms modal
    JoinRoom,
    /// Submit new room modal
    JoinOrCreateRoom,
    /// Submit new message
    SendMessage,
    /// Escape to rooms
    Escape,
}

impl Action {
    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::Sleep => &[Key::Ctrl('s')],
            Action::NewRoom => &[Key::Char('a')],
            Action::LeaveRoom => &[Key::Char('l')],
            Action::RoomUsers => &[Key::Char('u')],
            Action::Messages => &[Key::Char('m'), Key::Enter],
            Action::MaybeFocusNewMessage => &[Key::Char('M')],
            Action::FocusNewMessage => &[Key::Char('m'), Key::Enter],
            Action::Chats => &[Key::Char('p')],
            Action::AllUsers => &[Key::Char('U')],
            Action::AllRooms => &[Key::Char('R')],
            Action::ListPrev => &[Key::Char('k'), Key::Up],
            Action::ListNext => &[Key::Char('j'), Key::Down],
            Action::NewChat => &[Key::Char('m'), Key::Enter],
            Action::JoinRoom => &[Key::Enter],
            Action::JoinOrCreateRoom => &[Key::Enter],
            Action::SendMessage => &[Key::Enter],
            Action::Escape => &[Key::Esc],
        }
    }

    pub fn iterator() -> std::slice::Iter<'static, Action> {
        static ACTIONS: [Action; 18] = [
            Action::Quit,
            Action::Sleep,
            Action::NewRoom,
            Action::LeaveRoom,
            Action::RoomUsers,
            Action::Messages,
            Action::MaybeFocusNewMessage,
            Action::FocusNewMessage,
            Action::Chats,
            Action::AllUsers,
            Action::AllRooms,
            Action::ListPrev,
            Action::ListNext,
            Action::NewChat,
            Action::JoinRoom,
            Action::JoinOrCreateRoom,
            Action::SendMessage,
            Action::Escape,
        ];
        ACTIONS.iter()
    }

    pub fn display_with_keys(&self) -> String {
        let keys = self.keys().iter().map(|k| k.to_string()).collect::<Vec<_>>().join(", ");
        format!("{self}: {keys}")
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Action::Quit => "Quit",
            Action::Sleep => "Sleep",
            Action::NewRoom => "Join room",
            Action::LeaveRoom => "Leave room",
            Action::RoomUsers => "Room members",
            Action::Messages => "Room messages",
            Action::MaybeFocusNewMessage => "New message",
            Action::FocusNewMessage => "New message",
            Action::Chats => "Private chats",
            Action::AllUsers => "List all users",
            Action::AllRooms => "List all rooms",
            Action::ListPrev => "Previous",
            Action::ListNext => "Next",
            Action::NewChat => "Message user",
            Action::JoinRoom => "Join room",
            Action::JoinOrCreateRoom => "Join/Create room",
            Action::SendMessage => "Send",
            Action::Escape => "Escape",
        };
        write!(f, "{str}")
    }
}

#[derive(Debug)]
pub struct Actions(Vec<Action>);

impl Actions {
    pub fn find(&self, key: Key) -> Option<&Action> {
        Action::iterator()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    pub fn actions(&self) -> &[Action] {
        &self.0
    }
}

impl From<Vec<Action>> for Actions {
    fn from(actions: Vec<Action>) -> Self {
        Self(actions)
    }
}
