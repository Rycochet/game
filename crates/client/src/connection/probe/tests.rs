use super::*;

fn server() -> PortAnswer {
    PortAnswer::Server(ServerVersion::this())
}

#[test]
fn the_first_port_with_a_server_wins() {
    let ports = [8081, 443];
    let found = |port| {
        Some(ProbeResult::Found {
            port,
            version: ServerVersion::this(),
        })
    };
    assert_eq!(decide(&ports, &[Some(server()), None]), found(8081));
    assert_eq!(
        decide(&ports, &[Some(server()), Some(server())]),
        found(8081)
    );
    assert_eq!(
        decide(&ports, &[Some(PortAnswer::Silent), Some(server())]),
        found(443)
    );
    assert_eq!(
        decide(&ports, &[Some(PortAnswer::Other), Some(server())]),
        found(443)
    );
}

#[test]
fn a_later_server_waits_for_the_ports_before_it() {
    assert_eq!(decide(&[8081, 443], &[None, Some(server())]), None);
    assert_eq!(decide(&[8081, 443], &[None, None]), None);
}

#[test]
fn why_nothing_was_found() {
    let ports = [8081, 443];
    let silent = Some(PortAnswer::Silent);
    let other = Some(PortAnswer::Other);
    let refused = Some(PortAnswer::Refused("wrong key".into()));
    assert_eq!(
        decide(&ports, &[silent.clone(), silent.clone()]),
        Some(ProbeResult::NoAnswer)
    );
    assert_eq!(
        decide(&ports, &[silent.clone(), other.clone()]),
        Some(ProbeResult::NotAServer)
    );
    assert_eq!(
        decide(&ports, &[refused, other]),
        Some(ProbeResult::Refused("wrong key".into()))
    );
}

#[test]
fn answers_read() {
    let json = ServerVersion::this().to_json();
    assert_eq!(read_answer(200, &json), server());
    assert_eq!(read_answer(404, &json), PortAnswer::Other);
    assert_eq!(read_answer(200, "<html>hello</html>"), PortAnswer::Other);
}
