//! Starting Bevy's remote protocol, and adding our own methods to it.

use std::net::{Ipv4Addr, TcpListener};

use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use bevy_remote::http::RemoteHttpPlugin;
use bevy_remote::{RemoteMethodSystemId, RemoteMethods, RemotePlugin, error_codes};
use serde::de::DeserializeOwned;

pub use bevy_remote::{BrpError, BrpResult};
pub use serde_json::{Value, json};

/// Methods added before the remote protocol has started, put in place by
/// `register_methods` once every plugin is built. So a module may add its
/// methods whether its plugin comes before `DebugApiPlugin` or after.
#[derive(Resource, Default)]
struct PendingMethods(Vec<(String, Handler)>);

/// A method's system, as `RemotePlugin` keeps them.
type Handler = BoxedSystem<In<Option<Value>>, BrpResult>;

/// Adding a debug API method: `app.add_debug_method("client.state", handler)`,
/// where the handler is a system taking `In<Option<Value>>` (the request's
/// params) and returning a `BrpResult` (the answer, as JSON). It does nothing
/// unless the debug API is on. Name methods `<program or module>.<what>`;
/// `tmp.<what>` is for trying something out on a branch, and fails
/// `cargo xtask test`, so it is removed or renamed before merging.
pub trait AddDebugMethod {
    fn add_debug_method<M>(
        &mut self,
        name: &str,
        handler: impl IntoSystem<In<Option<Value>>, BrpResult, M>,
    ) -> &mut Self;
}

impl AddDebugMethod for App {
    fn add_debug_method<M>(
        &mut self,
        name: &str,
        handler: impl IntoSystem<In<Option<Value>>, BrpResult, M>,
    ) -> &mut Self {
        let system: Handler = Box::new(IntoSystem::into_system(handler));
        self.world_mut()
            .get_resource_or_init::<PendingMethods>()
            .0
            .push((name.to_owned(), system));
        self
    }
}

/// Reads a method's params as `T`. Missing params read as `{}`, so a type
/// whose fields are all optional (or `serde(default)`) needs none.
pub fn params<T: DeserializeOwned>(params: Option<Value>) -> BrpResult<T> {
    serde_json::from_value(params.unwrap_or_else(|| json!({})))
        .map_err(|error| refused(error.to_string()))
}

/// The answer to a request that can't be done as asked, saying why.
pub fn refused(message: impl Into<String>) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message: message.into(),
        data: None,
    }
}

/// Starts the remote protocol on `port`, on localhost only, unless that port
/// is taken.
pub(super) fn start(app: &mut App, port: u16) {
    // Bevy's server would fail quietly on its own thread, so try the port
    // first, while it can still be reported.
    if let Err(error) = TcpListener::bind((Ipv4Addr::LOCALHOST, port)) {
        warn!(
            "The debug API can't listen on port {port} ({error}): is another copy running? Starting without it"
        );
        return;
    }
    app.add_plugins((
        RemotePlugin::default(),
        RemoteHttpPlugin::default()
            .with_address(Ipv4Addr::LOCALHOST)
            .with_port(port),
    ));
    info!("Debug API on http://127.0.0.1:{port}");
}

/// Puts the methods modules added in place, if the remote protocol started.
pub(super) fn register_methods(app: &mut App) {
    let world = app.world_mut();
    let Some(pending) = world.remove_resource::<PendingMethods>() else {
        return;
    };
    if !world.contains_resource::<RemoteMethods>() {
        return;
    }
    for (name, system) in pending.0 {
        let id = world.register_boxed_system(system);
        world
            .resource_mut::<RemoteMethods>()
            .insert(name, RemoteMethodSystemId::Instant(id));
    }
}

#[cfg(test)]
mod tests;
