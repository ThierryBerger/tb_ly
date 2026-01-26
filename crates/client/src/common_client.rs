//! This module introduces a settings struct that can be used to configure the server and client.
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::net::{Ipv4Addr, SocketAddr};

use bevy::prelude::*;

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use lightyear::netcode::client_plugin::NetcodeConfig;
use lightyear::netcode::{ConnectToken, NetcodeClient};
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use shared::settings::SharedSettings;

/// Event that examples can trigger to spawn a client.
#[derive(Component, Clone, Debug)]
#[component(on_add = ExampleClient::on_add)]
pub struct ExampleClient {
    pub client_id: u64,
    /// The client port to listen on
    pub client_port: u16,
    /// The socket address of the server
    pub server_addr: SocketAddr,
    /// Possibly add a conditioner to simulate network conditions
    pub conditioner: Option<RecvLinkConditioner>,
    pub shared: SharedSettings,
}

impl ExampleClient {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<ExampleClient>().unwrap();
            let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client_port);
            entity_mut.insert((
                Client::default(),
                Link::new(settings.conditioner.clone()),
                LocalAddr(client_addr),
                PeerAddr(settings.server_addr),
                ReplicationReceiver::default(),
                PredictionManager::default(),
                Name::from("Client"),
            ));

            let certificate_digest = {
                #[cfg(target_family = "wasm")]
                {
                    include_str!("../../../certificates/digest.txt").to_string()
                }
                #[cfg(not(target_family = "wasm"))]
                {
                    include_str!("../../../certificates/digest.txt").to_string()
                }
            };
            entity_mut.insert(WebTransportClientIo { certificate_digest });

            Ok(())
        });
    }
}

pub(crate) fn connect(mut commands: Commands, client: Single<Entity, With<Client>>) {
    commands.trigger(Connect {
        entity: client.into_inner(),
    });
}
