use std::{
    error::Error,
    io::{self, stdout},
    net::ToSocketAddrs,
    sync::Arc,
    time::Duration,
};

use clap::Parser;
use client::{
    app::{App, AppReturn},
    inputs::{Events, InputEvent},
    io::{IoEvent, IoHandler},
    ui,
};
use common::{
    client::Client,
    commands::{Command, KEEP_ALIVE_CHECK, KEEP_ALIVE_INTERVAL},
};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedReceiver},
    Mutex,
};
use tui::{backend::CrosstermBackend, Terminal};

#[derive(Debug, Parser)]
#[command(author, version, long_about = None)]
/// Run an rs_chat client
struct Args {
    /// username to connect to server with
    #[arg(short, default_value = "guest")]
    user: String,
    /// host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
    /// port
    #[arg(short, default_value = "4000")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let addr = (args.host, args.port)
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;

    let client = Client::connect(addr, args.user.clone()).await?;
    let (io_tx, io_rx) = unbounded_channel();

    let app = App::new(io_tx);
    let app = Arc::new(Mutex::new(app));

    set_panic();
    start_io(client, app.clone(), io_rx).await;
    start_ui(app, args.user).await
}

async fn start_io(mut client: Client, app: Arc<Mutex<App>>, mut io_rx: UnboundedReceiver<IoEvent>) {
    client.hello().await.unwrap();
    let mut io_handler = IoHandler::new(client, app.clone());

    // Send keep alive
    let keep_alive_app = app.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(KEEP_ALIVE_INTERVAL)).await;

            let mut app = keep_alive_app.lock().await;
            // Comment out to see server kill connection if it doesn't get keep alive
            app.dispatch(IoEvent::Command(Command::KeepAlive));
        }
    });

    // Check keep alive
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(KEEP_ALIVE_CHECK)).await;

            let mut app = app.lock().await;
            if !app.state.keep_alive() {
                reset_terminal().unwrap();
                eprintln!("Server shutdown, quitting...");
                std::process::exit(1);
            } else {
                app.state.set_keep_alive(false);
            }
        }
    });

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

async fn start_ui(app: Arc<Mutex<App>>, username: String) -> Result<(), Box<dyn Error>> {
    let mut stdout = stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    let tick_rate = Duration::from_millis(200);
    let mut events = Events::new(tick_rate);

    loop {
        let mut app = app.lock().await;

        terminal.draw(|rect| ui::draw(rect, &mut app, &username))?;

        let result = match events.next().await {
            InputEvent::Input(key) => app.do_action(key, &username),
            InputEvent::Tick => app.update_on_tick(),
        };

        if result == AppReturn::Exit {
            break;
        }
    }

    reset_terminal()?;
    Ok(())
}

fn set_panic() {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        reset_terminal().unwrap();
        original_hook(panic)
    }));
}

fn reset_terminal() -> Result<(), Box<dyn Error>> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), LeaveAlternateScreen)?;

    Ok(())
}
