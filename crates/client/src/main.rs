use std::{error::Error, io::stdout, time::Duration, sync::Arc};

use client::{inputs::{Events, InputEvent}, app::{App, AppReturn}, ui};
use tokio::sync::Mutex;
use tui::{backend::CrosstermBackend, Terminal};

fn fake_vec(prefix: &str, count: usize) -> Vec<String> {
    (1..=count).into_iter().map(|i| format!("{prefix} {i}")).collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    app.active_rooms.items = fake_vec("Room", 20);
    app.room_users.items = fake_vec("User", 20);
    app.all_users.items = fake_vec("User", 20);
    app.all_rooms.items = fake_vec("Room", 20);

    let app = Arc::new(Mutex::new(app));
    start_ui(app).await
}

async fn start_ui(app: Arc<Mutex<App>>) -> Result<(), Box<dyn Error>> {
    crossterm::terminal::enable_raw_mode()?;

    let stdout = stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;
    terminal.hide_cursor()?;

    let tick_rate = Duration::from_millis(200);
    let mut events = Events::new(tick_rate);

    loop {
        let mut app = app.lock().await;

        terminal.draw(|rect| ui::draw(rect, &mut app))?;
        
        let result = match events.next().await {
            InputEvent::Input(key) => app.do_action(key),
            InputEvent::Tick => app.update_on_tick() 
        };

        if result == AppReturn::Exit {
            break;
        }
    }

    terminal.clear()?;
    terminal.show_cursor()?;

    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
