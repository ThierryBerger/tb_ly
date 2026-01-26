pub mod auth;
pub mod game;
pub mod protocol;
pub mod settings;

use avian2d::prelude::LinearVelocity;
use bevy::{math::VectorSpace, prelude::*};
use leafwing_input_manager::prelude::ActionState;
use lightyear::prelude::*;

use crate::{protocol::*, settings::FIXED_TIMESTEP_HZ};

#[derive(Clone)]
pub struct SharedPlugin;

impl Plugin for SharedPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ProtocolPlugin);
        app.add_plugins(game::plugin);
    }
}

// This system defines how we update the player's positions when we receive an input
pub fn shared_movement_behaviour(
    mut velocity: Mut<LinearVelocity>,
    action: &ActionState<PlayerActions>,
) {
    trace!(pressed = ?action.get_pressed(), "shared movement");
    const MOVE_SPEED: f32 = 10.0;
    const MAX_VELOCITY: f32 = 150.0;
    let mut change = Vec2::ZERO;
    if action.pressed(&PlayerActions::Up) {
        change.y += MOVE_SPEED;
    }
    if action.pressed(&PlayerActions::Down) {
        change.y -= MOVE_SPEED;
    }
    if action.pressed(&PlayerActions::Left) {
        change.x -= MOVE_SPEED;
    }
    if action.pressed(&PlayerActions::Right) {
        change.x += MOVE_SPEED;
    }

    fn move_toward_zero(value: f32, step: f32) -> f32 {
        if value.abs() <= step {
            0.0
        } else {
            value - value.signum() * step
        }
    }
    if change.x == 0f32 || (velocity.x != 0f32 && change.x.signum() != velocity.x.signum()) {
        velocity.x = move_toward_zero(velocity.x, MOVE_SPEED);
    }
    if change.y == 0f32 || (velocity.y != 0f32 && change.y.signum() != velocity.y.signum()) {
        velocity.y = move_toward_zero(velocity.y, MOVE_SPEED);
    }
    velocity.0 += change;
    *velocity = LinearVelocity(velocity.clamp_length_max(MAX_VELOCITY));
    //dbg!(velocity);
}

/// Generate a color from the `ClientId`
pub fn color_from_id(client_id: PeerId) -> Color {
    let h = (((client_id.to_bits().wrapping_mul(30)) % 360) as f32) / 360.0;
    let s = 1.0;
    let l = 0.5;
    Color::hsl(h, s, l)
}
