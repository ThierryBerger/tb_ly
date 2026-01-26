use avian2d::prelude::*;
use bevy::prelude::*;

pub const PLAYER_SIZE: f32 = 10f32;

#[derive(Bundle)]
pub struct PhysicsBundle {
    pub collider: Collider,
    pub collider_density: ColliderDensity,
    pub rigid_body: RigidBody,
    pub restitution: Restitution,
    pub mass: MassPropertiesBundle,
}

impl PhysicsBundle {
    pub fn player() -> Self {
        let collider = Collider::circle(PLAYER_SIZE);
        let mass = MassPropertiesBundle::from_shape(&collider, 0.1f32);
        Self {
            collider,
            collider_density: ColliderDensity(0.2),
            rigid_body: RigidBody::Dynamic,
            restitution: Restitution::new(0.3),
            mass,
        }
    }
}
