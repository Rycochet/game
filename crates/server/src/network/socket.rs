//! Carrying messages over one WebSocket: sending, pinging and logging trouble.

use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use bevy::prelude::*;
use shared::protocol::{ClientMessage, ServerMessage, wired};
use std::sync::atomic::Ordering;
use std::thread;
use std::time::{Duration, Instant};

use super::{MAX_MESSAGE, NO_ANSWER_AFTER, Sockets};
use crate::latency::{Latency, Pings};

/// A request to open a WebSocket.
pub(super) async fn upgrade(upgrade: WebSocketUpgrade, State(sockets): State<Sockets>) -> Response {
    upgrade
        .max_message_size(MAX_MESSAGE)
        .on_upgrade(move |socket| carry(socket, sockets))
}

/// Sends a message; `false` if it failed or the client isn't taking it.
async fn send(socket: &mut WebSocket, message: Message, limit: Duration) -> bool {
    matches!(
        tokio::time::timeout(limit, socket.send(message)).await,
        Ok(Ok(()))
    )
}

/// Logs a connection going into trouble or coming out of it.
pub(super) fn log_trouble(number: u64, latency: &Latency, troubled: bool) {
    let round_trip = latency.smoothed_ms().unwrap_or_default();
    let jitter = latency.jitter_ms();
    if troubled {
        warn!(
            "Connection {number} is in trouble: round trips {round_trip:.0} ms, varying by {jitter:.0} ms"
        );
    } else {
        info!(
            "Connection {number} is over its trouble: round trips {round_trip:.0} ms, varying by {jitter:.0} ms"
        );
    }
}

/// Carries one client's messages between its WebSocket and its link, until
/// either side goes or the client falls silent. Pings are answered here
/// instead, and this times its own.
async fn carry(mut socket: WebSocket, sockets: Sockets) {
    let (link, wire) = wired();
    if sockets.new_connections.send(link).is_err() {
        return;
    }
    let number = sockets.connections.fetch_add(1, Ordering::Relaxed) + 1;
    let limit = sockets.silent_after;
    let mut pings = Pings::new();
    let mut latency = Latency::default();
    let mut last_heard = Instant::now();
    let (to_link, from_link) = wire.into_parts();
    // `from_link` only has a waiting `recv`: a thread waits on it and passes
    // what the game sends to this task. It ends when the game drops the link.
    let (outgoing, mut to_socket) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    thread::spawn(move || {
        while let Ok(bytes) = from_link.recv() {
            if outgoing.send(bytes).is_err() {
                break;
            }
        }
    });
    let mut ping = tokio::time::interval(sockets.ping_every);
    // After a hold-up, carry on rather than sending the missed ones in a burst.
    ping.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                last_heard = Instant::now();
                match incoming {
                    Some(Ok(Message::Binary(bytes))) => {
                        if let Some(sent) = ClientMessage::read_ping(&bytes) {
                            let pong = ServerMessage::pong_bytes(sent, sockets.clock.minutes());
                            if !send(&mut socket, Message::Binary(pong.into()), limit).await {
                                break;
                            }
                        } else if to_link.send(bytes.to_vec()).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Pong(payload))) => {
                        if let Some(round_trip) = pings.answered(&payload, last_heard) {
                            debug!("Connection {number} ping: {:.1} ms", round_trip.as_secs_f64() * 1000.0);
                            if let Some(troubled) = latency.measured(round_trip) {
                                log_trouble(number, &latency, troubled);
                            }
                        }
                    }
                    // Text isn't part of the game; pings are answered by the
                    // library.
                    Some(Ok(Message::Text(_) | Message::Ping(_))) => {}
                    Some(Ok(Message::Close(_)) | Err(_)) | None => break,
                }
            }
            outgoing = to_socket.recv() => match outgoing {
                Some(bytes) => {
                    if !send(&mut socket, Message::Binary(bytes.into()), limit).await {
                        warn!("Connection {number} isn't taking what is sent: dropping it");
                        break;
                    }
                }
                // The game has dropped this client.
                None => {
                    send(&mut socket, Message::Close(None), limit).await;
                    break;
                }
            },
            _ = ping.tick() => {
                let now = Instant::now();
                if now.duration_since(last_heard) >= limit {
                    warn!("Connection {number} has gone silent: dropping it");
                    break;
                }
                if pings
                    .waiting_since()
                    .is_some_and(|sent| now.duration_since(sent) >= NO_ANSWER_AFTER)
                    && let Some(troubled) = latency.no_answer()
                {
                    log_trouble(number, &latency, troubled);
                }
                let payload = pings.next(now).to_vec();
                if !send(&mut socket, Message::Ping(payload.into()), limit).await {
                    break;
                }
            }
        }
    }
    if let Some(round_trip) = latency.smoothed_ms() {
        info!(
            "Connection {number} ended: round trips {round_trip:.0} ms, varying by {:.0} ms",
            latency.jitter_ms()
        );
    }
    // Dropping `to_link` tells the game the client has gone.
}
