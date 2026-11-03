use super::*;

use std::io::{Read, Write};
use std::net::TcpStream;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::ring::default_provider;
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use shared::protocol::PROTOCOL_VERSION;
use tungstenite::Message as WsMessage;

use rustls::sign::CertifiedKey;

use crate::lets_encrypt::DOMAIN_CHECK_PROTOCOL;
use crate::lets_encrypt::tests::expect_check;
use crate::storage::tests::TestFolder;
use crate::tls::CertificateSource;
use crate::tls::tests::lets_encrypt_certificates;

/// A web server with no game running: nothing ever reads its links.
fn serve_without_a_game(
    ping_every: Duration,
    silent_after: Duration,
) -> (SocketAddr, Receiver<ServerLink>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (new_connections, accepted) = mpsc::channel();
    let clock = WorldClock::new(405.0);
    let sockets = Sockets {
        new_connections,
        clock,
        connections: Arc::default(),
        ping_every,
        silent_after,
    };
    thread::spawn(move || serve(listener, None, None, sockets, Arc::default()));
    (address, accepted)
}

/// A server with TLS on and no game running: its plain and secure
/// addresses, its links, and the folder keeping its certificate.
fn serve_securely() -> (SocketAddr, SocketAddr, Receiver<ServerLink>, TestFolder) {
    let folder = TestFolder::new();
    let source = CertificateSource::Generated {
        folder: folder.0.join("tls"),
        names: Vec::new(),
    };
    let certificates = Arc::new(Certificates::load(&source).unwrap());
    let (plain, secure, accepted) = serve_with(certificates);
    (plain, secure, accepted, folder)
}

/// A server with no game running, using these certificates.
fn serve_with(certificates: Arc<Certificates>) -> (SocketAddr, SocketAddr, Receiver<ServerLink>) {
    serve_with_plain_too(certificates, false)
}

/// The same, saying whether the plain port serves the game as well.
fn serve_with_plain_too(
    certificates: Arc<Certificates>,
    plain_too: bool,
) -> (SocketAddr, SocketAddr, Receiver<ServerLink>) {
    let secure = SecurePort {
        listener: TcpListener::bind("127.0.0.1:0").unwrap(),
        certificates,
        ask_lets_encrypt: false,
        plain_too,
    };
    let secure_address = secure.listener.local_addr().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let plain_address = listener.local_addr().unwrap();
    let (new_connections, accepted) = mpsc::channel();
    let sockets = Sockets {
        new_connections,
        clock: WorldClock::new(405.0),
        connections: Arc::default(),
        ping_every: PING_EVERY,
        silent_after: SILENT_AFTER,
    };
    thread::spawn(move || serve(listener, Some(secure), None, sockets, Arc::default()));
    (plain_address, secure_address, accepted)
}

/// Certificates using Let's Encrypt, as if it were checking the domain now;
/// and the answer to the check.
fn expecting_a_check(folder: &TestFolder) -> (Arc<Certificates>, Arc<CertifiedKey>) {
    let certificates = lets_encrypt_certificates(&folder.0);
    let answer = expect_check(&certificates.issuing().unwrap());
    (Arc::new(certificates), answer)
}

/// A client with these protocols connecting by this name.
fn connect_as(
    secure: SocketAddr,
    name: &'static str,
    protocols: &[&[u8]],
) -> rustls::StreamOwned<rustls::ClientConnection, TcpStream> {
    let mut config = (*trusting_client()).clone();
    config.alpn_protocols = protocols.iter().map(|protocol| protocol.to_vec()).collect();
    let name = ServerName::try_from(name).unwrap();
    let connection = rustls::ClientConnection::new(Arc::new(config), name).unwrap();
    rustls::StreamOwned::new(connection, TcpStream::connect(secure).unwrap())
}

#[test]
fn a_domain_check_gets_its_answer_and_nothing_else_does() {
    let folder = TestFolder::new();
    let (certificates, answer) = expecting_a_check(&folder);
    let (_, secure, _accepted) = serve_with(certificates);

    // The check: the special certificate, then the connection closes.
    let mut check = connect_as(secure, "game.example.com", &[DOMAIN_CHECK_PROTOCOL]);
    let mut nothing = Vec::new();
    let _ = check.read_to_end(&mut nothing);
    assert!(nothing.is_empty());
    assert_eq!(check.conn.peer_certificates().unwrap()[0], answer.cert[0]);

    // Asking about another domain gets nowhere.
    let mut other = connect_as(secure, "other.example.com", &[DOMAIN_CHECK_PROTOCOL]);
    assert!(other.read_to_end(&mut nothing).is_err());

    // Anyone else gets the ordinary certificate and the game.
    let mut player = connect_as(secure, "game.example.com", &[b"http/1.1"]);
    let page = ask(&mut player, CERTIFICATE_PATH);
    assert!(page.starts_with("HTTP/1.1 200"), "{page}");
    assert_ne!(player.conn.peer_certificates().unwrap()[0], answer.cert[0]);
}

/// Accepts any certificate: these tests are about the server. The real
/// checks are the client's (`connection::trust`).
#[derive(Debug)]
struct AcceptAnything;

impl ServerCertVerifier for AcceptAnything {
    fn verify_server_cert(
        &self,
        _: &CertificateDer<'_>,
        _: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        default_provider()
            .signature_verification_algorithms
            .supported_schemes()
    }
}

fn trusting_client() -> Arc<rustls::ClientConfig> {
    Arc::new(
        rustls::ClientConfig::builder_with_provider(Arc::new(default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(AcceptAnything))
            .with_no_client_auth(),
    )
}

/// Asks for a path over HTTP on any stream; the whole answer.
fn ask(mut stream: impl Read + Write, path: &str) -> String {
    write!(
        stream,
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )
    .unwrap();
    let mut answer = Vec::new();
    // A peer closing without TLS's goodbye is fine here.
    let _ = stream.read_to_end(&mut answer);
    String::from_utf8_lossy(&answer).into_owned()
}

#[test]
fn behind_a_proxy_both_ports_serve_the_game() {
    let folder = TestFolder::new();
    let source = CertificateSource::Generated {
        folder: folder.0.join("tls"),
        names: Vec::new(),
    };
    let certificates = Arc::new(Certificates::load(&source).unwrap());
    let (plain, secure, accepted) = serve_with_plain_too(certificates, true);

    let (_plain_socket, _) = tungstenite::connect(format!("ws://{plain}{SOCKET_PATH}")).unwrap();
    assert!(accepted.recv_timeout(Duration::from_secs(5)).is_ok());

    let url = format!("wss://localhost:{}{SOCKET_PATH}", secure.port());
    let (_secure_socket, _) = tungstenite::client_tls_with_config(
        url,
        TcpStream::connect(secure).unwrap(),
        None,
        Some(tungstenite::Connector::Rustls(trusting_client())),
    )
    .unwrap();
    assert!(accepted.recv_timeout(Duration::from_secs(5)).is_ok());

    // The certificate is only asked for over a connection that uses it.
    let answer = ask(TcpStream::connect(plain).unwrap(), CERTIFICATE_PATH);
    assert!(answer.starts_with("HTTP/1.1 404"), "{answer}");
    let mut player = connect_as(secure, "localhost", &[b"http/1.1"]);
    let answer = ask(&mut player, CERTIFICATE_PATH);
    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
}

#[test]
fn the_plain_port_says_to_use_the_secure_one() {
    let (plain, secure, _accepted, _folder) = serve_securely();
    for path in ["/", SOCKET_PATH] {
        let answer = ask(TcpStream::connect(plain).unwrap(), path);
        assert!(answer.starts_with("HTTP/1.1 426"), "{answer}");
        assert!(
            answer.contains(&format!("port {}", secure.port())),
            "{answer}"
        );
    }
    let url = format!("ws://{plain}{SOCKET_PATH}");
    assert!(tungstenite::connect(url).is_err());
}

#[test]
fn the_certificate_can_be_asked_for_and_is_the_one_in_use() {
    let (_, secure, _accepted, folder) = serve_securely();
    let name = ServerName::try_from("localhost").unwrap();
    let connection = rustls::ClientConnection::new(trusting_client(), name).unwrap();
    let mut stream = rustls::StreamOwned::new(connection, TcpStream::connect(secure).unwrap());

    let answer = ask(&mut stream, CERTIFICATE_PATH);

    assert!(answer.starts_with("HTTP/1.1 200"), "{answer}");
    assert!(answer.contains("BEGIN CERTIFICATE"), "{answer}");
    assert!(!answer.contains("PRIVATE KEY"), "{answer}");
    let on_disk = std::fs::read_to_string(folder.0.join("tls/certificate.pem")).unwrap();
    let from_disk = CertificateDer::from_pem_slice(on_disk.as_bytes()).unwrap();
    assert_eq!(stream.conn.peer_certificates().unwrap()[0], from_disk);
}

#[test]
fn the_game_works_over_a_secure_websocket() {
    let (_, secure, accepted, _folder) = serve_securely();
    let url = format!("wss://localhost:{}{SOCKET_PATH}", secure.port());
    let stream = TcpStream::connect(secure).unwrap();
    let (mut socket, _) = tungstenite::client_tls_with_config(
        url,
        stream,
        None,
        Some(tungstenite::Connector::Rustls(trusting_client())),
    )
    .unwrap();
    // Kept: the socket closes when the link goes, maybe before the answer.
    let _link = accepted.recv_timeout(Duration::from_secs(5)).unwrap();

    socket
        .send(WsMessage::Binary(ClientMessage::ping_bytes(2.5).into()))
        .unwrap();
    let answer = loop {
        if let WsMessage::Binary(bytes) = socket.read().unwrap() {
            break bytes;
        }
    };
    assert_eq!(ServerMessage::read_pong(&answer).unwrap().0, 2.5);
}

#[test]
fn a_silent_client_is_dropped() {
    let (address, accepted) =
        serve_without_a_game(Duration::from_millis(50), Duration::from_millis(300));
    // Never read from, so it never answers a ping.
    let (_socket, _) = tungstenite::connect(format!("ws://{address}{SOCKET_PATH}")).unwrap();
    let link = accepted.recv_timeout(Duration::from_secs(5)).unwrap();

    let deadline = Instant::now() + Duration::from_secs(5);
    while link.receive() != Err(shared::protocol::LinkError::Closed) {
        assert!(Instant::now() < deadline, "the client wasn't dropped");
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn a_client_answering_pings_stays() {
    let (address, accepted) =
        serve_without_a_game(Duration::from_millis(50), Duration::from_millis(300));
    let (mut socket, _) = tungstenite::connect(format!("ws://{address}{SOCKET_PATH}")).unwrap();
    if let tungstenite::stream::MaybeTlsStream::Plain(stream) = socket.get_mut() {
        stream
            .set_read_timeout(Some(Duration::from_millis(10)))
            .unwrap();
    }
    let link = accepted.recv_timeout(Duration::from_secs(5)).unwrap();

    // Reading is all it takes: the library answers. Sending nothing else.
    let end = Instant::now() + Duration::from_secs(1);
    while Instant::now() < end {
        match socket.read() {
            Ok(_) => {}
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) => {}
            Err(error) => panic!("the client was dropped: {error}"),
        }
    }
    assert!(link.receive().is_ok());
}

#[test]
fn pings_are_answered_without_the_game_and_the_rest_reach_it() {
    let (address, accepted) = serve_without_a_game(PING_EVERY, SILENT_AFTER);
    let url = format!("ws://{address}{SOCKET_PATH}");
    let (mut socket, _) = tungstenite::connect(url).unwrap();

    socket
        .send(WsMessage::Binary(ClientMessage::ping_bytes(1.5).into()))
        .unwrap();
    // (The keep-alive ping comes first, on opening.)
    let answer = loop {
        if let WsMessage::Binary(bytes) = socket.read().unwrap() {
            break bytes;
        }
    };
    let (sent, clock) = ServerMessage::read_pong(&answer).unwrap();
    assert_eq!(sent, 1.5);
    assert!((405.0..465.0).contains(&clock), "{clock}");

    let hello = ClientMessage::Hello {
        protocol: PROTOCOL_VERSION,
    };
    let bytes = postcard_bytes(&hello);
    socket.send(WsMessage::Binary(bytes.into())).unwrap();
    let link = accepted.recv_timeout(Duration::from_secs(5)).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        match link.receive().unwrap() {
            Some(message) => {
                assert_eq!(message, hello, "the ping must not reach the game");
                break;
            }
            None if std::time::Instant::now() > deadline => panic!("no hello arrived"),
            None => thread::sleep(Duration::from_millis(1)),
        }
    }
}

/// A message as the client's link would send it.
fn postcard_bytes(message: &ClientMessage) -> Vec<u8> {
    let (link, wire) = wired::<ClientMessage, ServerMessage>();
    link.send(message);
    wire.take().unwrap().unwrap()
}
