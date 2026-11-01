//! The world's clock: game minutes since the world began, running in real time
//! whatever the server is doing, as it is worked out from when it was last set
//! rather than added to each tick. The network thread answers pings from it,
//! and its own thread saves it. It can be stopped (`pause`), for single player:
//! anything going by the world's time should go by this.

use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use shared::world::GAME_MINUTE_SECONDS;

/// A clock any thread can read. Cloning it gives another handle on the same
/// clock (`Arc` shares one value between threads, `Mutex` lets them change it).
#[derive(Clone)]
pub(crate) struct WorldClock(Arc<Mutex<Set>>);

/// The time it was last set to, when, and whether it has stopped there.
#[derive(Clone, Copy)]
struct Set {
    minutes: f64,
    at: Instant,
    paused: bool,
}

impl Set {
    fn minutes(&self) -> f64 {
        if self.paused {
            self.minutes
        } else {
            self.minutes + self.at.elapsed().as_secs_f64() / GAME_MINUTE_SECONDS as f64
        }
    }
}

impl WorldClock {
    pub fn new(minutes: f64) -> Self {
        Self(Arc::new(Mutex::new(Set {
            minutes,
            at: Instant::now(),
            paused: false,
        })))
    }

    pub fn minutes(&self) -> f64 {
        self.0.lock().expect("never poisoned").minutes()
    }

    /// Moves it to another time, which can be earlier. Not before the world began.
    /// If it has stopped, it stays stopped there.
    pub fn set(&self, minutes: f64) {
        let mut set = self.0.lock().expect("never poisoned");
        set.minutes = minutes.max(0.0);
        set.at = Instant::now();
    }

    /// Stops it where it is, until `resume`.
    pub fn pause(&self) {
        self.set_paused(true);
    }

    /// Starts it again from where it stopped.
    pub fn resume(&self) {
        self.set_paused(false);
    }

    fn set_paused(&self, paused: bool) {
        let mut set = self.0.lock().expect("never poisoned");
        *set = Set {
            minutes: set.minutes(),
            at: Instant::now(),
            paused,
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
    fn a_paused_clock_stands_still_and_carries_on_from_there() {
        let clock = WorldClock::new(100.0);
        clock.pause();
        let stopped = clock.minutes();
        thread::sleep(Duration::from_millis(20));
        assert_eq!(clock.minutes(), stopped);
        // Twice makes no difference, and being moved doesn't start it.
        clock.pause();
        assert_eq!(clock.minutes(), stopped);
        clock.set(200.0);
        assert_eq!(clock.minutes(), 200.0);

        clock.resume();
        assert!(clock.minutes() < 200.0 + 1.0);
        thread::sleep(Duration::from_millis(20));
        assert!(clock.minutes() > 200.0);
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
