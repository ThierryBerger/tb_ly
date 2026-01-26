use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use leafwing_input_manager::prelude::InputMap;
use lightyear::prelude::*;

use shared::protocol::physics::PhysicsBundle;
use shared::protocol::*;
use shared::shared_movement_behaviour;

pub struct ExampleClientPlugin;

impl Plugin for ExampleClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, movement);
        app.add_observer(on_connected);
        app.add_observer(handle_predicted_spawn);
    }
}

fn on_connected(
    trigger: On<Add, Connected>,
    mut message_sender: Single<(Entity, &Client, &mut MessageSender<JoinGame>)>,
) {
    dbg!(trigger.entity);
    message_sender.2.send::<ChannelPreGame>(JoinGame);
}

pub(crate) fn movement(
    // TODO: maybe make prediction mode a separate component!!!
    mut position_query: Query<(&mut LinearVelocity, &ActionState<PlayerActions>), With<Predicted>>,
) {
    for (velocity, action_state) in position_query.iter_mut() {
        //if !action_state.get_pressed().is_empty() {
        &action_state;
        shared_movement_behaviour(velocity, action_state);
        //}
    }
}

/// When the predicted copy of the client-owned entity is spawned, do stuff
/// - assign it a different saturation
/// - keep track of it in the Global resource
pub(crate) fn handle_predicted_spawn(
    trigger: On<Add, (PlayerId, Predicted)>,
    mut predicted: Query<(&mut ColorComponent, Has<Controlled>), (With<Predicted>, With<PlayerId>)>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    if let Ok((mut color, controlled)) = predicted.get_mut(entity) {
        let hsva = Hsva {
            saturation: 0.4,
            ..Hsva::from(color.0)
        };
        color.0 = Color::from(hsva);
        warn!("Add InputMarker to entity: {:?}", entity);

        let mut entity_mut = commands.entity(entity);
        entity_mut.insert(PhysicsBundle::player());
        if controlled {
            entity_mut.insert(InputMap::new([
                (PlayerActions::Up, KeyCode::KeyW),
                (PlayerActions::Down, KeyCode::KeyS),
                (PlayerActions::Left, KeyCode::KeyA),
                (PlayerActions::Right, KeyCode::KeyD),
            ]));
        }
    }
}
