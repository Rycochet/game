use super::*;

fn asked() -> ServerAddress {
    ServerAddress::parse("play.example.com").unwrap()
}

#[test]
fn a_server_found_is_at_the_port_that_answered() {
    let result = ProbeResult::Found {
        port: FALLBACK_PORT,
        version: ServerVersion::this(),
    };
    let found = checked(&asked(), &[DEFAULT_PORT, FALLBACK_PORT], result);
    let Checked::Found { address, .. } = found else {
        panic!("{found:?}");
    };
    assert_eq!(address.with_port(), "play.example.com:443");
}

#[test]
fn another_protocol_is_no_use() {
    let mut version = ServerVersion::this();
    version.protocol += 1;
    let result = ProbeResult::Found {
        port: 8081,
        version,
    };
    let Checked::Failed(reason) = checked(&asked(), &[8081], result) else {
        panic!("not failed");
    };
    assert!(reason.contains("newer"), "{reason}");
}

#[test]
fn reasons_name_the_ports_tried() {
    let both = checked(
        &asked(),
        &[DEFAULT_PORT, FALLBACK_PORT],
        ProbeResult::NoAnswer,
    );
    assert!(
        matches!(&both, Checked::Failed(reason) if reason.starts_with("No answer on port 8081 or 443"))
    );
    let one = checked(&asked(), &[9000], ProbeResult::NoAnswer);
    assert!(
        matches!(&one, Checked::Failed(reason) if reason.starts_with("No answer on port 9000"))
    );
}

#[test]
fn ready_only_for_the_text_checked() {
    let check = AddressCheck {
        of: Some(("play.example.com".into(), None)),
        checked: Checked::Found {
            address: asked(),
            version: ServerVersion::this(),
        },
        ..default()
    };
    assert!(check.ready_for("play.example.com").is_some());
    assert!(check.ready_for("play.example.co").is_none());
}
