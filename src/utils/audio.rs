use rodio::source::SineWave;
use rodio::{OutputStream, OutputStreamHandle, Sink, Source};
use std::cell::RefCell;
use std::io::{self, Write};
use std::time::Duration;

struct AudioEngine {
    // Keep stream alive for the lifetime of the process.
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl AudioEngine {
    fn new() -> Option<Self> {
        let (stream, handle) = OutputStream::try_default().ok()?;
        Some(Self {
            _stream: stream,
            handle,
        })
    }

    fn play_prestige_ready_tone(&self) -> bool {
        let Ok(sink) = Sink::try_new(&self.handle) else {
            return false;
        };

        // Short ascending chime to signal prestige readiness.
        let tone_a = SineWave::new(880.0)
            .take_duration(Duration::from_millis(75))
            .amplify(0.12);
        let tone_b = SineWave::new(1320.0)
            .take_duration(Duration::from_millis(110))
            .amplify(0.10);

        sink.append(tone_a);
        sink.append(tone_b);
        sink.detach();
        true
    }

    fn play_toggle_enabled_tone(&self) -> bool {
        let Ok(sink) = Sink::try_new(&self.handle) else {
            return false;
        };

        // Brief confirmation chirp when sound alerts are enabled.
        let tone_a = SineWave::new(660.0)
            .take_duration(Duration::from_millis(60))
            .amplify(0.10);
        let tone_b = SineWave::new(990.0)
            .take_duration(Duration::from_millis(80))
            .amplify(0.10);

        sink.append(tone_a);
        sink.append(tone_b);
        sink.detach();
        true
    }
}

std::thread_local! {
    static AUDIO_ENGINE: RefCell<Option<AudioEngine>> = const { RefCell::new(None) };
}

fn ring_terminal_bell() {
    // ANSI bell; terminal may ignore this depending on user/system settings.
    let _ = io::stdout().write_all(b"\x07");
    let _ = io::stdout().flush();
}

pub fn play_prestige_ready_sound() {
    let played = AUDIO_ENGINE.with(|engine_cell| {
        let mut engine = engine_cell.borrow_mut();
        if engine.is_none() {
            *engine = AudioEngine::new();
        }
        engine
            .as_ref()
            .map(|audio| audio.play_prestige_ready_tone())
            .unwrap_or(false)
    });

    if !played {
        ring_terminal_bell();
    }
}

pub fn play_sound_enabled_confirmation() {
    let played = AUDIO_ENGINE.with(|engine_cell| {
        let mut engine = engine_cell.borrow_mut();
        if engine.is_none() {
            *engine = AudioEngine::new();
        }
        engine
            .as_ref()
            .map(|audio| audio.play_toggle_enabled_tone())
            .unwrap_or(false)
    });

    if !played {
        ring_terminal_bell();
    }
}
