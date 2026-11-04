//! The server's clock: real seconds the server has run, worked out from when it
//! was last set rather than added to each tick, so it runs whatever the server
//! is doing. Every world's time is worked out from it (`shared::world::WorldTime`),
//! the network thread answers pings from it, and its own thread saves it. It
//! can be stopped (`pause`), for single player: anything going by the server's
//! or a world's time should go by this.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// A clock any thread can read. Cloning it gives another handle on the same
/// clock (`Arc` shares one value between threads, `Mutex` lets them change it).
#[derive(Clone)]
pub(crate) struct ServerClock(Arc<Mutex<Set>>);

/// The reading it was last set to, when, and whether it has stopped there.
#[derive(Clone, Copy)]
struct Set {
    seconds: f64,
    at: Instant,
    paused: bool,
}

impl Set {
    fn seconds(&self) -> f64 {
        if self.paused {
            self.seconds
        } else {
            self.seconds + self.at.elapsed().as_secs_f64()
        }
    }
}

impl ServerClock {
    pub fn new(seconds: f64) -> Self {
        Self(Arc::new(Mutex::new(Set {
            seconds,
            at: Instant::now(),
            paused: false,
        })))
    }

    pub fn seconds(&self) -> f64 {
        self.0.lock().expect("never poisoned").seconds()
    }

    /// Stops it where it is, until `resume`.
    pub fn pause(&self) {
        self.set_paused(true);
    }

    /// Starts it again from where it stopped.
    pub fn resume(&self) {
        self.set_paused(false);
    }

    pub fn is_paused(&self) -> bool {
        self.0.lock().expect("never poisoned").paused
    }

    fn set_paused(&self, paused: bool) {
        let mut set = self.0.lock().expect("never poisoned");
        *set = Set {
            seconds: set.seconds(),
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
mod tests;

/// Ticks of action time: what creatures' energy goes by (`shared::energy`), one
/// for the whole server. Apart from `ServerClock` as it has to be a whole number.
/// It stops when that is paused, and when the server is off.
#[derive(Clone, Default)]
pub struct ActionClock(Arc<AtomicU64>);

impl ActionClock {
    pub fn new(ticks: u64) -> Self {
        Self(Arc::new(AtomicU64::new(ticks)))
    }

    pub fn ticks(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }

    pub fn advance(&self, ticks: u64) {
        self.0.fetch_add(ticks, Ordering::Relaxed);
    }
}
