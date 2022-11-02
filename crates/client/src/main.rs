use std::{error::Error, io::stdout, net::SocketAddr, sync::Arc, time::Duration};

use clap::Parser;
use client::{
    app::{App, AppReturn},
    inputs::{Events, InputEvent},
    io::{IoEvent, IoHandler},
    ui,
};
use common::client::Client;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver},
    Mutex,
};
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
/// Run an irc client
struct Args {
    #[arg(short, default_value = "127.0.0.1:4000")]
    /// address of server
    address: SocketAddr,
    #[arg(short, default_value = "guest")]
    user: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let client = Client::connect(args.address, args.user.clone()).await?;
    let (io_tx, io_rx) = unbounded_channel();

    let app = App::new(io_tx);
    let app = Arc::new(Mutex::new(app));

    start_io(client, app.clone(), io_rx).await;
    start_ui(app, &args.user).await
}

async fn start_io(mut client: Client, app: Arc<Mutex<App>>, mut io_rx: UnboundedReceiver<IoEvent>) {
    client.hello().await.unwrap();
    let mut io_handler = IoHandler::new(client, app);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                maybe_response = io_handler.read_response() => {
                    let response = match maybe_response {
                        Ok(response) => response,
                        Err(_err) => break
                    };
                    io_handler.handle_response(response).await;
                }
                Some(event) = io_rx.recv() => {
                    io_handler.handle_io(event).await;
                }
            };
        }
    });
}

async fn start_ui(app: Arc<Mutex<App>>, username: &str) -> Result<(), Box<dyn Error>> {
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

        terminal.draw(|rect| ui::draw(rect, &mut app, username))?;

        let result = match events.next().await {
            InputEvent::Input(key) => app.do_action(key),
            InputEvent::Tick => app.update_on_tick(),
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
