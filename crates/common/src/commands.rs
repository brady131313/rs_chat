use serde::{Deserialize, Serialize};

use crate::Error;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Target {
    Username(String),
    Room(String),
}

pub const KEEP_ALIVE_INTERVAL: u64 = 5;
pub const KEEP_ALIVE_CHECK: u64 = 10;

#[derive(Debug)]
pub struct KeepAlive;

#[derive(Debug)]
pub struct Kill;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Command {
    Hello {
        username: String,
    },
    KeepAlive,
    ListRooms,
    ListUsers,
    JoinOrCreate {
        room: String,
    },
    Leave {
        room: String,
    },
    Send {
        target: Target,
        message: String,
    },
}

impl From<Command> for String {
    fn from(command: Command) -> Self {
        serde_json::to_string(&command).unwrap()
    }
}

impl<'a> TryFrom<&'a str> for Command {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        serde_json::from_str(value).map_err(|_| Error::InvalidCommand(value.into()))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Response {
    ListMembers { room: String, users: Vec<String> },
    ListRooms { rooms: Vec<String> },
    ListUsers { users: Vec<String> },
    TellRoom { 
        room: String,
        sender: String,
        message: String
    },
    TellUser {
        username: String,
        sender: String,
        message: String
    },
    KeepAlive,
    Err(ResponseError),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum ResponseError {
    UserAlreadyExists(String),
    RoomDoesNotExist(String),
    UserNotInRoom {
        user: String,
        room: String
    }
}

impl From<Response> for String {
    fn from(res: Response) -> Self {
        serde_json::to_string(&res).unwrap()
    }
}

impl<'a> TryFrom<&'a str> for Response {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        serde_json::from_str(value).map_err(|_| Error::InvalidResponse(value.into()))
    }
}
