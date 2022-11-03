use std::{sync::Arc, time::Duration};

use common::{
    client::Client,
    commands::{Command, Response},
    Result,
};
use tokio::sync::Mutex;

use crate::app::App;

pub enum IoEvent {
    Sleep,
    Command(Command),
}

pub struct IoHandler {
    client: Client,
    app: Arc<Mutex<App>>,
}

impl IoHandler {
    pub fn new(client: Client, app: Arc<Mutex<App>>) -> Self {
        Self { client, app }
    }

    pub async fn read_response(&mut self) -> Result<Response> {
        self.client.read_response().await
    }

    pub async fn handle_response(&mut self, response: Response) {
        match response {
            Response::ListMembers { room, users } => {
                let mut app = self.app.lock().await;
                app.add_active_room(room.clone());
                app.room_users_mut(&room).unwrap().items = users;
            }
            Response::ListUsers { users } => {
                let mut app = self.app.lock().await;
                app.all_users.items = users;
            }
            Response::ListRooms { rooms } => {
                let mut app = self.app.lock().await;
                app.all_rooms.items = rooms;
            }
            Response::TellRoom {
                room,
                sender,
                message,
            } => {
                let mut app = self.app.lock().await;
                app.room_messages_mut(&room)
                    .unwrap()
                    .items
                    .push((sender, message));
            }
            Response::TellUser {
                username,
                sender,
                message,
            } => {
                let mut app = self.app.lock().await;
                if self.client.username() == sender {
                    if app.chat_messages_mut(&username).is_none() {
                        app.add_chat(username.clone());
                    }

                    app.chat_messages_mut(&username)
                        .unwrap()
                        .items
                        .push((sender, message));
                } else {
                    if app.chat_messages_mut(&sender).is_none() {
                        app.add_chat(sender.clone());
                    }

                    app.chat_messages_mut(&sender)
                        .unwrap()
                        .items
                        .push((sender, message));
                }
            }
            Response::KeepAlive => {
                let mut app = self.app.lock().await;
                app.set_keep_alive(true);
            }
            Response::Err(_) => todo!(),
        }
    }

    pub async fn handle_io(&mut self, event: IoEvent) {
        match event {
            IoEvent::Sleep => self.handle_sleep().await,
            IoEvent::Command(command) => self.client.write_command(command).await.unwrap(),
        }
    }

    async fn handle_sleep(&self) {
        println!("sleeping");
        tokio::time::sleep(Duration::from_secs(5)).await;
        println!("Done sleeping")
    }
}
