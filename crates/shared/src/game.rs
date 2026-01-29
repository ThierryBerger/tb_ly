pub mod map;

use avian2d::{
    PhysicsPlugins,
    prelude::{
        Collider, ColliderDensity, Gravity, Mass, MassPropertiesBundle, PhysicsInterpolationPlugin,
        PhysicsTransformPlugin, Restitution, RigidBody,
    },
};
use bevy::prelude::*;
use lightyear::avian2d::plugin::AvianReplicationMode;

use crate::protocol::{ColorComponent, physics::PhysicsBundle};

pub fn plugin(app: &mut App) {
    app.add_plugins(lightyear::avian2d::plugin::LightyearAvianPlugin {
        replication_mode: AvianReplicationMode::Position,
        ..default()
    });
    app.add_plugins(
        PhysicsPlugins::default()
            .build()
            // disable the position<>transform sync plugins as it is handled by lightyear_avian
            .disable::<PhysicsTransformPlugin>()
            .disable::<PhysicsInterpolationPlugin>(),
    )
    .insert_resource(Gravity(Vec2::ZERO));

    app.add_systems(Startup, init_walls);
}

pub(crate) fn init_walls(mut commands: Commands) {
    map::create_map_3(commands.reborrow());
}

// Wall
#[derive(Bundle)]
pub struct WallBundle {
    physics: PhysicsBundle,
    wall: Wall,
    name: Name,
    color: ColorComponent,
}

#[derive(Component, Debug)]
pub struct Wall {
    pub start: Vec2,
    pub end: Vec2,
}

impl WallBundle {
    pub fn new(start: Vec2, end: Vec2, color: Color) -> Self {
        let collider = Collider::segment(start, end);
        let mass = MassPropertiesBundle::from_shape(&collider, 1f32);
        Self {
            physics: PhysicsBundle {
                collider,
                collider_density: ColliderDensity(1.0),
                rigid_body: RigidBody::Static,
                restitution: Restitution::new(0.0),
                mass,
            },
            wall: Wall { start, end },
            name: Name::from("Wall"),
            color: ColorComponent(color),
        }
    }
}
