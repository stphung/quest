//! Backend utilities for web streaming.
//!
//! Provides a TeeWriter that writes to both stdout and optionally broadcasts to WebSocket clients.

use std::io::{self, Stdout, Write};
use tokio::sync::broadcast;

/// A writer that writes to stdout and optionally broadcasts to WebSocket clients.
pub struct TeeWriter {
    /// The underlying stdout writer
    stdout: Stdout,
    /// Optional broadcast sender for WebSocket clients
    ws_sender: Option<broadcast::Sender<Vec<u8>>>,
    /// Buffer to batch writes before broadcasting
    buffer: Vec<u8>,
}

impl TeeWriter {
    /// Create a TeeWriter that broadcasts to WebSocket clients
    pub fn new(ws_sender: broadcast::Sender<Vec<u8>>) -> Self {
        Self {
            stdout: io::stdout(),
            ws_sender: Some(ws_sender),
            buffer: Vec::with_capacity(8192),
        }
    }

    /// Create a TeeWriter that only writes to stdout (no WebSocket)
    pub fn stdout_only() -> Self {
        Self {
            stdout: io::stdout(),
            ws_sender: None,
            buffer: Vec::new(),
        }
    }
}

impl Write for TeeWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // Write to real stdout
        let n = self.stdout.write(buf)?;

        // Buffer for WebSocket broadcast if enabled
        if self.ws_sender.is_some() {
            self.buffer.extend_from_slice(&buf[..n]);
        }

        Ok(n)
    }

    fn flush(&mut self) -> io::Result<()> {
        // Flush stdout
        self.stdout.flush()?;

        // Broadcast buffered data to WebSocket clients
        if let Some(ref sender) = self.ws_sender {
            if !self.buffer.is_empty() {
                // Ignore send errors (no subscribers is fine)
                // Use take() to avoid clone - replaces buffer with empty Vec
                let _ = sender.send(std::mem::take(&mut self.buffer));
            }
        }

        Ok(())
    }
}
