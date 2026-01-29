//! This module introduces a settings struct that can be used to configure the server and client.
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::net::{Ipv4Addr, SocketAddr};

use aeronet_webtransport::wtransport::Identity;
use bevy::prelude::*;
use core::time::Duration;

#[cfg(not(target_family = "wasm"))]
use async_compat::Compat;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
#[cfg(not(target_family = "wasm"))]
use bevy::tasks::IoTaskPool;
use lightyear::netcode::{NetcodeServer, PRIVATE_KEY_BYTES};
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use shared::settings::{SERVER_PORT, SharedSettings};
use tracing::warn;

use crate::auth::PRIVATE_KEY;
use crate::certificate::WebTransportCertificateSettings;

#[derive(Component, Debug)]
#[component(on_add = ExampleServer::on_add)]
pub struct ExampleServer {
    pub shared: SharedSettings,
}

impl ExampleServer {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<ExampleServer>().unwrap();
            entity_mut.insert((Name::from("Server"),));

            let add_netcode = |entity_mut: &mut EntityWorldMut| {
                // Use private key from settings file.
                let private_key = *PRIVATE_KEY;
                entity_mut.insert(NetcodeServer::new(NetcodeConfig {
                    protocol_id: settings.shared.protocol_id,
                    private_key,
                    ..Default::default()
                }));
            };
            add_netcode(&mut entity_mut);
            let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), SERVER_PORT);
            entity_mut.insert((
                LocalAddr(server_addr),
                WebTransportServerIo {
                    certificate: (&WebTransportCertificateSettings::FromFile {
                        cert: "../../certificates/cert.pem".to_string(),
                        key: "../../certificates/key.pem".to_string(),
                    })
                        .into(),
                },
            ));
            Ok(())
        });
    }
}

pub(crate) fn start(mut commands: Commands, server: Single<Entity, With<Server>>) {
    commands.trigger(Start {
        entity: server.into_inner(),
    });
}
