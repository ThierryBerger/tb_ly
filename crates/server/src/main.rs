//! This example showcases how to use Lightyear with Bevy, to easily get replication along with prediction/interpolation working.
//!
//! There is a lot of setup code, but it's mostly to have the examples work in all possible configurations of transport.
//! (all transports are supported, as well as running the example in client-and-server or host-server mode)
//!
//!
//! Run with
//! - `cargo run -- server`
//! - `cargo run -- client -c 1`

mod auth;
mod certificate;
mod common_server;
mod game;

use bevy::diagnostic::DiagnosticsPlugin;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use core::time::Duration;
use game::GameServerPlugin;
use lightyear::prelude::server::ServerPlugins;
use shared::SharedPlugin;
use shared::auth::AUTH_BACKEND_ADDRESS;
use shared::settings::{FIXED_TIMESTEP_HZ, SERVER_ADDR, SHARED_SETTINGS};
use tracing::Level;

use crate::common_server::*;

/// When running the example as a binary, we only support Client or Server mode.
fn main() {
    let mut app = new_headless_app();
    app.add_plugins(ServerPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
    });

    app.add_plugins(SharedPlugin);

    app.world_mut().spawn(ExampleServer {
        shared: SHARED_SETTINGS,
    });
    app.add_systems(Startup, start);

    app.add_plugins(GameServerPlugin);
    app.add_plugins(auth::AuthServerPlugin {
        game_server_addr: SERVER_ADDR,
        auth_backend_addr: AUTH_BACKEND_ADDRESS,
    });

    app.run();
}

pub fn log_plugin() -> LogPlugin {
    LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_enhanced_input::action::fns=error".to_string(),
        ..default()
    }
}

pub fn new_headless_app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        bevy::prelude::AssetPlugin::default(),
        bevy::scene::ScenePlugin,
        log_plugin(),
        StatesPlugin,
        DiagnosticsPlugin,
    ));
    app
}
