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
mod client;
mod common_client;
mod renderer;

use bevy::log::{Level, LogPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy::winit::WinitSettings;
use bevy_simple_prefs::{Prefs, PrefsPlugin};
use core::time::Duration;
use lightyear::link::RecvLinkConditioner;
use lightyear::prelude::client::InputDelayConfig;
use lightyear::prelude::*;
use shared::SharedPlugin;
use shared::auth::TokenResponse;
use shared::settings::{
    AUTH_SERVER_ADDR, CLIENT_PORT, FIXED_TIMESTEP_HZ, GAME_SERVER_ADDR, SHARED_SETTINGS,
};

use crate::auth::AuthClientPlugin;
use crate::client::ExampleClientPlugin;
use crate::common_client::ExampleClient;

#[derive(Resource, Reflect, Clone, Default)]
struct AuthPrefs {
    pub accounts: Vec<AuthTokenDef>,
    pub current: Option<usize>,
}

impl AuthPrefs {
    fn get_current_auth(&self) -> Option<&AuthTokenDef> {
        let current = self.current?;
        self.accounts.get(current)
    }

    fn get_current_auth_mut(&mut self) -> Option<&mut AuthTokenDef> {
        let current = self.current?;
        self.accounts.get_mut(current)
    }
}

/// Represents an identity, isolated to allow storing multiple accounts on a same preference.
#[derive(Resource, Reflect, Clone, Default)]
struct AuthTokenDef {
    pub client_id: Option<u64>,
    pub last_token: Option<Vec<u8>>,
    pub secret: String,
}

#[derive(Reflect, Prefs, Default)]
struct MyPrefs {
    pub tokens: AuthPrefs,
}

/// When running the example as a binary, we only support Client or Server mode.
fn main() {
    let mut app = new_gui_app();
    app.add_plugins((lightyear::prelude::client::ClientPlugins {
        tick_duration: Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ),
    },));
    app.add_plugins(SharedPlugin);
    app.add_plugins(AuthClientPlugin {
        auth_backend_address: AUTH_SERVER_ADDR.parse().unwrap(),
    });
    app.add_plugins(PrefsPlugin::<MyPrefs>::default());
    app.world_mut()
        .spawn(ExampleClient {
            client_port: CLIENT_PORT,

            server_addr: GAME_SERVER_ADDR.parse().unwrap(),
            conditioner: Some(RecvLinkConditioner::new(LinkConditionerConfig {
                incoming_latency: Duration::from_secs_f32(0.15),
                incoming_jitter: Duration::from_secs_f32(0.1),
                incoming_loss: 0.02,
            })),
            shared: SHARED_SETTINGS,
        })
        .insert(
            InputTimelineConfig::default().with_input_delay(InputDelayConfig::fixed_input_delay(3)),
        );

    //app.add_systems(Startup, connect);
    app.add_plugins(ExampleClientPlugin);

    //#[cfg(feature = "gui")]
    app.add_plugins(renderer::ExampleRendererPlugin);

    app.run();
}

pub fn new_gui_app() -> App {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .build()
            .set(AssetPlugin {
                // https://github.com/bevyengine/bevy/issues/10157
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            })
            .set(log_plugin())
            .set(window_plugin()),
    );
    // we want the same frequency of updates for both focused and unfocused
    // Otherwise when testing the movement can look choppy for unfocused windows
    app.insert_resource(WinitSettings::continuous());

    #[cfg(feature = "debug")]
    {
        app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
        app.add_plugins(DebugUIPlugin);
    }
    //app.add_plugins(bevy_inspector_egui::bevy_egui::EguiPlugin::default());
    //app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
    app
}

pub fn window_plugin() -> WindowPlugin {
    WindowPlugin {
        primary_window: Some(Window {
            title: format!("Lightyear Example: {}", env!("CARGO_PKG_NAME")),
            resolution: (1024, 768).into(),
            present_mode: PresentMode::AutoVsync,
            // set to true if we want to capture tab etc in wasm
            prevent_default_event_handling: true,
            ..Default::default()
        }),
        ..default()
    }
}

pub fn log_plugin() -> LogPlugin {
    LogPlugin {
        level: Level::INFO,
        filter: "wgpu=error,bevy_render=info,bevy_ecs=warn,bevy_time=warn,naga=warn,bevy_enhanced_input::action::fns=error".to_string(),
        ..default()
    }
}
