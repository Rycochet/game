use super::*;

#[test]
fn answers_are_timed_and_kept_for_the_game() {
    let latest = LatestPong::default();
    let pinger = Pinger::new(latest.clone());
    let ping = pinger.ping();
    let sent = ClientMessage::read_ping(&ping).unwrap();
    std::thread::sleep(Duration::from_millis(5));

    assert!(pinger.took_pong(&ServerMessage::pong_bytes(sent, 6.75)));

    let pong = latest.take().unwrap();
    assert!(pong.round_trip >= 0.005);
    assert_eq!(pong.clock, 6.75);
    assert!(latest.take().is_none());
}

#[test]
fn other_messages_are_left_for_the_link() {
    let latest = LatestPong::default();
    let pinger = Pinger::new(latest.clone());
    assert!(!pinger.took_pong(&[1, 2, 3]));
    assert!(latest.take().is_none());
}

#[test]
fn only_the_latest_answer_is_kept() {
    let latest = LatestPong::default();
    let pinger = Pinger::new(latest.clone());
    pinger.took_pong(&ServerMessage::pong_bytes(0.0, 1.0));
    pinger.took_pong(&ServerMessage::pong_bytes(0.0, 2.0));
    assert_eq!(latest.take().unwrap().clock, 2.0);
}
