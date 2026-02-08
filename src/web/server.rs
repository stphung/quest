//! WebSocket server for streaming terminal output to browsers.

use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio_tungstenite::tungstenite::Message;

/// Channel for sending terminal output to all connected clients
pub type OutputSender = broadcast::Sender<Vec<u8>>;

/// Channel for receiving keyboard input from any client
pub type InputReceiver = mpsc::Receiver<crossterm::event::KeyEvent>;
pub type InputSender = mpsc::Sender<crossterm::event::KeyEvent>;

/// The web server state
#[allow(dead_code)]
pub struct WebServer {
    /// Broadcast channel for terminal output
    pub output_tx: OutputSender,
    /// Channel for receiving input from web clients
    input_rx: Arc<Mutex<InputReceiver>>,
    /// Sender for input (cloned for each connection)
    pub input_tx: InputSender,
}

impl WebServer {
    /// Create a new web server
    pub fn new() -> Self {
        let (output_tx, _) = broadcast::channel(100);
        let (input_tx, input_rx) = mpsc::channel(100);

        Self {
            output_tx,
            input_rx: Arc::new(Mutex::new(input_rx)),
            input_tx,
        }
    }

    /// Get a clone of the output sender for the tee backend
    #[allow(dead_code)]
    pub fn output_sender(&self) -> OutputSender {
        self.output_tx.clone()
    }

    /// Try to receive input from web clients (non-blocking, sync version)
    #[allow(dead_code)]
    pub fn try_recv_input_sync(&self) -> Option<crossterm::event::KeyEvent> {
        // Use try_lock to avoid blocking
        if let Ok(mut rx) = self.input_rx.try_lock() {
            rx.try_recv().ok()
        } else {
            None
        }
    }
}

impl Default for WebServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Start the web server on the given port
pub async fn start_web_server(port: u16, server: Arc<WebServer>) -> std::io::Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;

    eprintln!("Web server listening on ws://localhost:{}", port);
    eprintln!("Open http://localhost:{} in your browser", port);

    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let server = Arc::clone(&server);
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, addr, server).await {
                        eprintln!("Connection error from {}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                eprintln!("Accept error: {}", e);
            }
        }
    }
}

/// Handle a single WebSocket connection
async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    server: Arc<WebServer>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Check if this is a regular HTTP request or WebSocket upgrade
    let mut buf = [0u8; 4];
    let n = stream.peek(&mut buf).await?;

    if n >= 3 && &buf[..3] == b"GET" {
        // Peek more to check the path
        let mut peek_buf = [0u8; 256];
        let n = stream.peek(&mut peek_buf).await?;
        let request = String::from_utf8_lossy(&peek_buf[..n]);

        if request.contains("GET / ") && !request.contains("Upgrade: websocket") {
            // Serve the HTML page
            serve_html(stream).await?;
            return Ok(());
        }
    }

    // WebSocket upgrade
    let ws_stream = tokio_tungstenite::accept_async(stream).await?;
    eprintln!("WebSocket connection from: {}", addr);

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Subscribe to terminal output
    let mut output_rx = server.output_tx.subscribe();

    // Clone input sender for this connection
    let input_tx = server.input_tx.clone();

    // Spawn task to send terminal output to this client
    let send_task = tokio::spawn(async move {
        loop {
            match output_rx.recv().await {
                Ok(data) => {
                    if ws_sender.send(Message::Binary(data)).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Closed) => break,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    });

    // Handle incoming messages (keyboard input)
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                // Parse keyboard input from JSON
                if let Ok(key_event) = parse_key_event(&text) {
                    let _ = input_tx.send(key_event).await;
                }
            }
            Ok(Message::Close(_)) => break,
            Err(_) => break,
            _ => {}
        }
    }

    send_task.abort();
    eprintln!("WebSocket disconnected: {}", addr);

    Ok(())
}

/// Serve the HTML page for the web terminal
async fn serve_html(mut stream: TcpStream) -> std::io::Result<()> {
    use tokio::io::AsyncWriteExt;

    let html = include_str!("../../web/index.html");
    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        html.len(),
        html
    );

    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

/// Parse a key event from JSON sent by the browser
fn parse_key_event(json: &str) -> Result<crossterm::event::KeyEvent, ()> {
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};

    // Simple JSON parsing for key events
    // Format: {"key": "a"} or {"key": "Enter"} or {"key": "ArrowUp"}
    let json = json.trim();
    if !json.starts_with('{') || !json.ends_with('}') {
        return Err(());
    }

    // Extract the key value
    let key = json
        .split("\"key\"")
        .nth(1)
        .and_then(|s| s.split('"').nth(1))
        .ok_or(())?;

    let code = match key {
        "Enter" => KeyCode::Enter,
        "Escape" => KeyCode::Esc,
        "Backspace" => KeyCode::Backspace,
        "Tab" => KeyCode::Tab,
        "ArrowUp" | "Up" => KeyCode::Up,
        "ArrowDown" | "Down" => KeyCode::Down,
        "ArrowLeft" | "Left" => KeyCode::Left,
        "ArrowRight" | "Right" => KeyCode::Right,
        " " => KeyCode::Char(' '),
        s if s.len() == 1 => KeyCode::Char(s.chars().next().unwrap()),
        _ => return Err(()),
    };

    Ok(KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
        state: KeyEventState::empty(),
    })
}
