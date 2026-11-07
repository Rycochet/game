//! Asking this network's router to forward the open port here (UPnP), so
//! players on the internet can reach a player's own game, or a standalone
//! server with `HOW_UPNP` (`upnp_plugin`), without setting up port forwarding
//! by hand.
//!
//! It runs on a thread of its own, as finding the router can take seconds:
//! it finds the router, forwards the port for an hour at a time (renewing it
//! while open, or for good if the router only allows that), and asks for the
//! address the internet sees. Dropping `RouterWork` removes the forwarding.
//! Each step's outcome is a `RouterPort` for the host.

use bevy::log::{info, warn};
use igd_next::aio::Gateway;
use igd_next::aio::tokio::{Tokio, search_gateway};
use igd_next::{AddPortError, PortMappingProtocol, SearchOptions};
use shared::protocol::RouterPort;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::Mutex;
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::sync::watch;

/// How long to wait for a router to answer.
const SEARCH_FOR: Duration = Duration::from_secs(10);
/// How long each forwarding lasts, in seconds, unless renewed: a game that
/// stops without removing it leaves it for at most this long.
const LEASE: u32 = 60 * 60;
/// How often the forwarding is renewed while open.
const RENEW_EVERY: Duration = Duration::from_secs(30 * 60);
/// How long removing the forwarding may take on closing.
const REMOVE_FOR: Duration = Duration::from_secs(3);
/// What the router lists the forwarding as.
const DESCRIPTION: &str = "Hall of Worlds";

/// The router being asked to forward a port; dropping it stops and removes
/// the forwarding.
pub(super) struct RouterWork {
    /// Dropping it tells the thread to stop.
    stop: Option<watch::Sender<()>>,
    thread: Option<JoinHandle<()>>,
    answers: Mutex<mpsc::Receiver<RouterPort>>,
}

impl RouterWork {
    /// Starts asking the router to forward its `external` port to this
    /// computer's port `local` listens on (all its addresses if unspecified).
    pub(super) fn start(external: u16, local: SocketAddr) -> Self {
        let (port, bind) = (local.port(), local.ip());
        let (stop, stopped) = watch::channel(());
        let (tell, answers) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("upnp".into())
            .spawn(move || run(external, port, bind, tell, stopped))
            .ok();
        Self {
            stop: Some(stop),
            thread,
            answers: Mutex::new(answers),
        }
    }

    /// The latest news from the router since last asked, if any.
    pub(super) fn latest(&self) -> Option<RouterPort> {
        let answers = self.answers.lock().ok()?;
        answers.try_iter().last()
    }
}

impl Drop for RouterWork {
    fn drop(&mut self) {
        self.stop.take();
        if let Some(thread) = self.thread.take() {
            // So the forwarding is gone before the port is opened again.
            let _ = thread.join();
        }
    }
}

fn run(
    external: u16,
    port: u16,
    bind: IpAddr,
    tell: mpsc::Sender<RouterPort>,
    stopped: watch::Receiver<()>,
) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    match runtime {
        Ok(runtime) => runtime.block_on(forward(external, port, bind, tell, stopped)),
        Err(error) => {
            let _ = tell.send(RouterPort::Failed(format!("can't start ({error})")));
        }
    }
}

async fn forward(
    external: u16,
    port: u16,
    bind: IpAddr,
    tell: mpsc::Sender<RouterPort>,
    mut stopped: watch::Receiver<()>,
) {
    let fail = |reason: String| {
        warn!("Can't forward port {external} on the router: {reason}");
        let _ = tell.send(RouterPort::Failed(reason));
    };
    if bind.is_loopback() {
        return fail("only listening on this computer, which the router can't reach".into());
    }
    let mut options = SearchOptions::default();
    options.timeout = Some(SEARCH_FOR);
    let found = tokio::select! {
        found = search_gateway(options) => found,
        _ = stopped.changed() => return,
    };
    let Ok(gateway) = found else {
        return fail("no router answered (it may not have UPnP, or it is turned off)".into());
    };
    let Some(local) = lan_address(bind, gateway.addr) else {
        return fail("can't tell this computer's address on the network".into());
    };
    let to = SocketAddr::new(local, port);
    let lease = match add(&gateway, external, to).await {
        Ok(lease) => lease,
        Err(reason) => return fail(reason),
    };
    let answer = match gateway.get_external_ip().await {
        Ok(ip) => RouterPort::Open {
            external: SocketAddr::new(ip, external).to_string(),
            shared: is_shared(ip),
        },
        // Forwarded all the same.
        Err(_) => RouterPort::Open {
            external: String::new(),
            shared: false,
        },
    };
    info!("The router forwards port {external} to {to}: {answer:?}");
    let _ = tell.send(answer);
    loop {
        tokio::select! {
            _ = stopped.changed() => break,
            _ = tokio::time::sleep(RENEW_EVERY), if lease > 0 => {
                if let Err(reason) = add(&gateway, external, to).await {
                    warn!("Can't renew port {external} on the router: {reason}");
                }
            }
        }
    }
    let removing = gateway.remove_port(PortMappingProtocol::TCP, external);
    match tokio::time::timeout(REMOVE_FOR, removing).await {
        Ok(Ok(())) => info!("The router no longer forwards port {external}"),
        _ => warn!("Can't remove port {external} from the router; it lapses within the hour"),
    }
}

/// Forwards the router's `port` to `to`, for `LEASE` or, if the router only
/// allows that, for good; the lease it got (0 for good), or why not.
async fn add(gateway: &Gateway<Tokio>, port: u16, to: SocketAddr) -> Result<u32, String> {
    let tcp = PortMappingProtocol::TCP;
    match gateway.add_port(tcp, port, to, LEASE, DESCRIPTION).await {
        Ok(()) => Ok(LEASE),
        Err(AddPortError::OnlyPermanentLeasesSupported) => {
            match gateway.add_port(tcp, port, to, 0, DESCRIPTION).await {
                Ok(()) => Ok(0),
                Err(error) => Err(refusal(error)),
            }
        }
        Err(error) => Err(refusal(error)),
    }
}

/// Why the router said no, in players' words.
fn refusal(error: AddPortError) -> String {
    match error {
        AddPortError::PortInUse => {
            "the router already forwards that port to another computer".into()
        }
        AddPortError::ActionNotAuthorized => "the router doesn't allow it".into(),
        other => format!("the router refused ({other})"),
    }
}

/// This computer's address on the router's network: the one listened on, or
/// for all of them, the one the router is reached from.
pub(super) fn lan_address(bind: IpAddr, router: SocketAddr) -> Option<IpAddr> {
    if !bind.is_unspecified() {
        return Some(bind);
    }
    let any: SocketAddr = match router {
        SocketAddr::V4(_) => ([0, 0, 0, 0], 0).into(),
        SocketAddr::V6(_) => (std::net::Ipv6Addr::UNSPECIFIED, 0).into(),
    };
    // Nothing is sent: connecting a UDP socket only picks the route.
    let socket = UdpSocket::bind(any).ok()?;
    socket.connect(router).ok()?;
    Some(socket.local_addr().ok()?.ip())
}

/// Whether an address the router has on the internet is really a private or
/// shared one (the internet provider's own network, CGNAT), which players
/// outside still can't reach.
pub(super) fn is_shared(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, ..] = ip.octets();
            let carrier_grade = a == 100 && (64..128).contains(&b);
            ip.is_private()
                || carrier_grade
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_unspecified()
        }
        IpAddr::V6(ip) => ip.is_loopback() || ip.is_unspecified(),
    }
}

#[cfg(test)]
mod tests;
