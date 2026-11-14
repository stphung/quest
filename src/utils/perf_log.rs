//! Opt-in frame-timing log for diagnosing UI lag (issue #815).
//!
//! Set `QUEST_PERF_LOG=/path/to/file` and the game loops write one line per
//! loop phase: `<ms since start>\t<phase>\t<micros spent in phase>`.
//! A phase's time is the time since the previous mark, so a slow draw, a slow
//! input handler, or a slow tick each show up under their own label.
//!
//! Phases in the game loop: `update` (ticks, saves, housekeeping), `draw`,
//! `wait` (idle time blocked on the next key or frame deadline — expected to
//! be up to ~50ms), `input` (handling a key). The splash/character-select
//! loop uses `splash_update` (including key handling), `splash_list`,
//! `splash_draw`, `splash_wait`. Anything other than a `*wait` phase taking
//! more than a few milliseconds is the lag.
//! With the variable unset, `mark` is a single failed env lookup the first
//! time and a cheap flag check afterwards.

use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

/// Environment variable naming the file to write timings to.
pub const PERF_LOG_ENV: &str = "QUEST_PERF_LOG";

/// Writes phase timings to any sink. The global [`mark`] wraps one of these
/// around a buffered file.
pub struct PerfLog<W: Write> {
    out: W,
    started: Instant,
    last: Instant,
}

impl<W: Write> PerfLog<W> {
    /// Start a log, writing a `#` header line describing the build.
    pub fn new(mut out: W, header: &str) -> Self {
        let now = Instant::now();
        let _ = writeln!(out, "# {header}");
        let _ = writeln!(out, "# ms_since_start\tphase\tmicros");
        Self {
            out,
            started: now,
            last: now,
        }
    }

    /// Record that `phase` just finished; its duration is the time since the
    /// previous mark (or since the log started).
    pub fn mark(&mut self, phase: &str) {
        self.mark_at(phase, Instant::now());
    }

    fn mark_at(&mut self, phase: &str, now: Instant) {
        let since_start = now.duration_since(self.started).as_millis();
        let spent = now.duration_since(self.last).as_micros();
        let _ = writeln!(self.out, "{since_start}\t{phase}\t{spent}");
        let _ = self.out.flush();
        self.last = now;
    }

    #[cfg(test)]
    fn into_inner(self) -> W {
        self.out
    }
}

type FileLog = PerfLog<std::io::BufWriter<std::fs::File>>;

fn global() -> &'static Option<Mutex<FileLog>> {
    static LOG: OnceLock<Option<Mutex<FileLog>>> = OnceLock::new();
    LOG.get_or_init(|| {
        let path = std::env::var_os(PERF_LOG_ENV)?;
        let file = std::fs::File::create(path).ok()?;
        let header = format!(
            "quest {} ({})",
            super::build_info::BUILD_COMMIT,
            super::build_info::BUILD_DATE
        );
        Some(Mutex::new(PerfLog::new(
            std::io::BufWriter::new(file),
            &header,
        )))
    })
}

/// Record a finished loop phase in the perf log, if `QUEST_PERF_LOG` is set.
pub fn mark(phase: &str) {
    if let Some(log) = global() {
        if let Ok(mut log) = log.lock() {
            log.mark(phase);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn writes_header_then_one_line_per_phase_with_time_since_previous_mark() {
        let mut log = PerfLog::new(Vec::new(), "quest test-build");
        let t0 = log.started;
        log.mark_at("draw", t0 + Duration::from_millis(3));
        log.mark_at("input", t0 + Duration::from_millis(53));

        let text = String::from_utf8(log.into_inner()).unwrap();
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "# quest test-build");
        assert_eq!(lines[1], "# ms_since_start\tphase\tmicros");
        assert_eq!(lines[2], "3\tdraw\t3000");
        assert_eq!(lines[3], "53\tinput\t50000");
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn mark_measures_from_log_start_for_the_first_phase() {
        let mut log = PerfLog::new(Vec::new(), "h");
        log.mark("tick");
        let text = String::from_utf8(log.into_inner()).unwrap();
        let fields: Vec<&str> = text.lines().nth(2).unwrap().split('\t').collect();
        assert_eq!(fields[1], "tick");
        assert!(fields[0].parse::<u128>().is_ok());
        assert!(fields[2].parse::<u128>().is_ok());
    }

    #[test]
    fn global_mark_is_a_no_op_without_the_env_var() {
        // The test process doesn't set QUEST_PERF_LOG; this must not panic
        // or create anything.
        mark("draw");
    }
}
