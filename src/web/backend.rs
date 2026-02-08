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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tee_writer_buffers_before_flush() {
        let (tx, mut rx) = broadcast::channel(10);
        let mut writer = TeeWriter::new(tx);

        // Write without flushing - should buffer but not broadcast yet
        let _ = writer.write(b"hello");

        // Try to receive - should fail since we haven't flushed
        assert!(rx.try_recv().is_err());

        // Buffer should contain the data
        assert_eq!(writer.buffer, b"hello");
    }

    #[test]
    fn test_tee_writer_broadcasts_on_flush() {
        let (tx, mut rx) = broadcast::channel(10);
        let mut writer = TeeWriter::new(tx);

        // Write and flush
        let _ = writer.write(b"hello");
        let _ = writer.flush();

        // Should receive the data
        let received = rx.try_recv().unwrap();
        assert_eq!(received, b"hello");

        // Buffer should be cleared after flush
        assert!(writer.buffer.is_empty());
    }

    #[test]
    fn test_tee_writer_multiple_writes_before_flush() {
        let (tx, mut rx) = broadcast::channel(10);
        let mut writer = TeeWriter::new(tx);

        // Multiple writes
        let _ = writer.write(b"hello");
        let _ = writer.write(b" ");
        let _ = writer.write(b"world");
        let _ = writer.flush();

        // Should receive concatenated data
        let received = rx.try_recv().unwrap();
        assert_eq!(received, b"hello world");
    }

    #[test]
    fn test_tee_writer_stdout_only_does_not_buffer() {
        let mut writer = TeeWriter::stdout_only();

        // Write to stdout-only writer
        let _ = writer.write(b"hello");

        // Buffer should remain empty (no WebSocket to broadcast to)
        assert!(writer.buffer.is_empty());
    }

    #[test]
    fn test_tee_writer_empty_flush_does_not_broadcast() {
        let (tx, mut rx) = broadcast::channel(10);
        let mut writer = TeeWriter::new(tx);

        // Flush without writing anything
        let _ = writer.flush();

        // Should not receive anything
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn test_tee_writer_no_subscribers_ok() {
        let (tx, _rx) = broadcast::channel::<Vec<u8>>(10);
        // Drop the receiver to simulate no subscribers
        drop(_rx);

        let mut writer = TeeWriter::new(tx);

        // Write and flush should not panic even with no subscribers
        let _ = writer.write(b"hello");
        let result = writer.flush();
        assert!(result.is_ok());
    }
}
