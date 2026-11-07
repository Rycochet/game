use super::*;

#[test]
fn private_and_carrier_addresses_are_shared() {
    for shared in [
        "192.168.1.1",
        "10.0.0.5",
        "172.16.0.1",
        "100.64.0.1",
        "100.127.255.255",
    ] {
        assert!(is_shared(shared.parse().unwrap()), "{shared}");
    }
    for public in ["203.0.113.5", "100.128.0.1", "8.8.8.8", "2001:db8::1"] {
        assert!(!is_shared(public.parse().unwrap()), "{public}");
    }
}

#[test]
fn a_chosen_listen_address_is_the_one_forwarded_to() {
    let router: SocketAddr = "192.168.1.1:1900".parse().unwrap();
    let bind: IpAddr = "192.168.1.20".parse().unwrap();
    assert_eq!(lan_address(bind, router), Some(bind));
}

#[test]
fn listening_only_here_never_asks_a_router() {
    let work = RouterWork::start(24691, "127.0.0.1".parse().unwrap());
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    let answer = loop {
        if let Some(answer) = work.latest() {
            break answer;
        }
        assert!(std::time::Instant::now() < deadline, "no answer");
        std::thread::sleep(Duration::from_millis(10));
    };
    assert!(matches!(answer, RouterPort::Failed(reason) if reason.contains("only listening")));
}
