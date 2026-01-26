use avian2d::prelude::{LinearVelocity, Position, Rotation, SweptCcd};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use lightyear::connection::client::PeerMetadata;
use lightyear::prelude::*;
use shared::protocol::physics::PhysicsBundle;
use shared::protocol::*;
use shared::settings::SEND_INTERVAL;
use shared::{color_from_id, shared_movement_behaviour};

const OBSTACLE_GAP: f32 = 50.0;
const OBSTACLES_ROW_COL: i32 = 50;
const INTEREST_RADIUS: f32 = 150.0;

// Plugin for server-specific logic
pub struct GameServerPlugin;

impl Plugin for GameServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RoomPlugin);
        app.add_systems(Startup, init);

        // the physics/FixedUpdates systems that consume inputs should be run in this set
        app.add_systems(FixedUpdate, movement);
        // messages are not subject to a particular schedule
        app.add_systems(Update, handle_join_game);
        app.add_observer(handle_new_client);
        app.add_systems(Update, interest_management);

        // Spawn a room for the player entities
        app.world_mut().spawn(Room::default());
    }
}

/// When a new client tries to connect to a server, an entity is created for it with the `ClientOf` component.
/// This entity represents the connection between the server and that client.
///
/// You can add additional components to update the connection. In this case we will add a `ReplicationSender` that
/// will enable us to replicate local entities to that client.
pub(crate) fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(ReplicationSender::new(
            SEND_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ));
}

/// If the new client connects to the server, we want to spawn a new player entity for it.
///
/// We can't react before `Connected` because there is no guarantee that the connection request we
/// received was valid. The server could reject the connection attempt for many reasons (server is full, packet is invalid,
/// DDoS attempt, etc.). We want to start the replication only when the client is confirmed as connected.
///
/// We're reading a specific [`JoinGame`] message to allow specific tuning of gameplay (choose type of player...)
/// FIXME: this may have to be back to [`On<Add, Connected>`], as player gameplay type should be known before and not changeable here.
pub(crate) fn handle_join_game(
    receiver: Query<(Entity, &RemoteId, &mut MessageReceiver<JoinGame>)>,
    room: Single<Entity, With<Room>>,
    mut commands: Commands,
) {
    for (e, client_id, mut message) in receiver {
        let client_id = client_id.0;
        message.receive().for_each(|_message| {
            let color = color_from_id(client_id);
            let player_entity = commands
                .spawn((
                    PlayerId(client_id),
                    Position(Vec2::ZERO),
                    Rotation::default(),
                    LinearVelocity::ZERO,
                    ColorComponent(color),
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
                    ControlledBy {
                        owner: e,
                        lifetime: Default::default(),
                    },
                    PhysicsBundle::player(),
                    SweptCcd::default(),
                    // Use network visibility for interest management
                    NetworkVisibility,
                ))
                .id();
            info!(
                "Create entity {:?} for client {:?}",
                player_entity, client_id
            );

            let room = room.entity();
            // we can control the player visibility in a more static manner by using rooms
            // we add all clients to a room, as well as all player entities
            // this means that all clients will be able to see all player entities
            commands.trigger(RoomEvent {
                target: RoomTarget::AddSender(e),
                room,
            });
            commands.trigger(RoomEvent {
                target: RoomTarget::AddEntity(player_entity),
                room,
            });
            // TODO: avoid multiple spawns
        })
    }
}

pub(crate) fn init(mut commands: Commands) {
    // spawn dots in a grid
    for x in -OBSTACLES_ROW_COL..OBSTACLES_ROW_COL {
        for y in -OBSTACLES_ROW_COL..OBSTACLES_ROW_COL {
            commands.spawn((
                Position(Vec2::new(x as f32 * OBSTACLE_GAP, y as f32 * OBSTACLE_GAP)),
                CircleMarker,
                Replicate::to_clients(NetworkTarget::All),
                NetworkVisibility,
            ));
        }
    }
}

/// Here we perform more "immediate" interest management: we will make a circle visible to a client
/// depending on the distance to the client's entity
pub(crate) fn interest_management(
    peer_metadata: Res<PeerMetadata>,
    player_query: Query<(&PlayerId, Ref<Position>), (Without<CircleMarker>, With<Replicate>)>,
    mut circle_query: Query<
        (Entity, &Position, &mut ReplicationState),
        (With<CircleMarker>, With<Replicate>, With<NetworkVisibility>),
    >,
) {
    for (client_id, position) in player_query.iter() {
        let Some(sender_entity) = peer_metadata.mapping.get(&client_id.0) else {
            error!("Could not find sender entity for client: {:?}", client_id);
            return;
        };
        if position.is_changed() {
            // in real game, you would have a spatial index (kd-tree) to only find entities within a certain radius
            for (circle, circle_position, mut state) in circle_query.iter_mut() {
                let distance = position.distance(**circle_position);
                if distance < INTEREST_RADIUS {
                    trace!("Gain visibility with {circle:?}");
                    state.gain_visibility(*sender_entity);
                } else {
                    trace!("Lose visibility with {circle:?}");
                    state.lose_visibility(*sender_entity);
                }
            }
        }
    }
}

/// Read client inputs and move players
/// NOTE: this system can now be run in both client/server!
pub(crate) fn movement(
    timeline: Res<LocalTimeline>,
    mut action_query: Query<(
        Entity,
        &Position,
        &mut LinearVelocity,
        &ActionState<PlayerActions>,
    )>,
) {
    let tick = timeline.tick();
    for (entity, position, velocity, action) in action_query.iter_mut() {
        //if !action.get_pressed().is_empty() {
        // NOTE: be careful to directly pass Mut<PlayerPosition>
        // getting a mutable reference triggers change detection, unless you use `as_deref_mut()`
        shared_movement_behaviour(velocity, action);
        trace!(?entity, ?tick, ?position, actions = ?action.get_pressed(), "applying movement to player");
        // }
    }
}
