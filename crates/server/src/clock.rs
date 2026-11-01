//! The world's clock: game minutes since the world began, running in real time
//! whatever the server is doing, as it is worked out from when it was last set
//! rather than added to each tick. The network thread answers pings from it,
//! and its own thread saves it.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use shared::world::GAME_MINUTE_SECONDS;

/// A clock any thread can read. Cloning it gives another handle on the same
/// clock (`Arc` shares one value between threads, `Mutex` lets them change it).
#[derive(Clone)]
pub(crate) struct WorldClock(Arc<Mutex<Set>>);

/// The time it was last set to, and when.
#[derive(Clone, Copy)]
struct Set {
    minutes: f64,
    at: Instant,
}

impl WorldClock {
    pub fn new(minutes: f64) -> Self {
        Self(Arc::new(Mutex::new(Set {
            minutes,
            at: Instant::now(),
        })))
    }

    pub fn minutes(&self) -> f64 {
        let set = *self.0.lock().expect("never poisoned");
        set.minutes + set.at.elapsed().as_secs_f64() / GAME_MINUTE_SECONDS as f64
    }

    /// Moves it to another time, which can be earlier. Not before the world began.
    pub fn set(&self, minutes: f64) {
        *self.0.lock().expect("never poisoned") = Set {
            minutes: minutes.max(0.0),
            at: Instant::now(),
        };
    }
}

/// Calls something at intervals on its own thread, so nothing the game is doing
/// can hold it up, and once more on being dropped. Dropping waits for that.
pub(crate) struct Repeating {
    /// Dropping this tells the thread to finish.
    stop: Option<Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Repeating {
    pub fn start(every: Duration, mut action: impl FnMut() + Send + 'static) -> Self {
        let (stop, stopped) = mpsc::channel::<()>();
        let thread = thread::Builder::new()
            .name("clock saver".into())
            .spawn(move || {
                while stopped.recv_timeout(every) == Err(RecvTimeoutError::Timeout) {
                    action();
                }
                action();
            })
            .expect("the clock's thread can start");
        Self {
            stop: Some(stop),
            thread: Some(thread),
        }
    }
}

impl Drop for Repeating {
    fn drop(&mut self) {
        drop(self.stop.take());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[test]
    fn the_clock_runs_and_can_be_moved_either_way() {
        let clock = WorldClock::new(100.0);
        let other = clock.clone();
        thread::sleep(Duration::from_millis(20));
        assert!(clock.minutes() > 100.0 && clock.minutes() < 110.0);
        other.set(30.0);
        assert!(clock.minutes() >= 30.0 && clock.minutes() < 40.0);
        other.set(-5.0);
        assert!(clock.minutes() >= 0.0 && clock.minutes() < 10.0);
    }

    #[test]
    fn it_repeats_and_runs_once_more_on_stopping() {
        let count = Arc::new(AtomicU32::new(0));
        let counted = count.clone();
        let repeating = Repeating::start(Duration::from_millis(5), move || {
            counted.fetch_add(1, Ordering::Relaxed);
        });
        thread::sleep(Duration::from_millis(50));
        let so_far = count.load(Ordering::Relaxed);
        assert!(so_far >= 1);
        drop(repeating);
        assert!(count.load(Ordering::Relaxed) > so_far);
    }
}
