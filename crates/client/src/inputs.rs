use std::time::Duration;

use crossterm::event::{Event, EventStream};
use futures::{FutureExt, StreamExt};
use futures_timer::Delay;

use self::key::Key;

pub mod key;
pub mod stateful_list;

pub enum InputEvent {
    Input(Key),
    Tick,
}

pub struct Events {
    stream: EventStream,
    tick_rate: Duration,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Self {
        Self {
            stream: EventStream::new(),
            tick_rate,
        }
    }

    pub async fn next(&mut self) -> InputEvent {
        let delay = Delay::new(self.tick_rate).fuse();
        let event = self.stream.next().fuse();

        tokio::select! {
            _ = delay => InputEvent::Tick,
            Some(Ok(Event::Key(key))) = event => InputEvent::Input(Key::from(key))
        }
    }
}
