use std::{sync::Arc, time::Duration};

use tokio::sync::Mutex;

use crate::app::App;

pub enum IoEvent {
    Sleep,
}

pub struct IoHandler {
    app: Arc<Mutex<App>>,
}

impl IoHandler {
    pub fn new(app: Arc<Mutex<App>>) -> Self {
        Self { app }
    }

    pub async fn handle_io(&self, event: IoEvent) {
        match event {
            IoEvent::Sleep => self.handle_sleep().await
        }
    }

    async fn handle_sleep(&self) {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
