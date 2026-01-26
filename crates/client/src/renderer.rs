use avian2d::prelude::{Position, Rotation};
use bevy::color::palettes::basic::GREEN;
use bevy::prelude::*;
use lightyear::prelude::*;
use lightyear_frame_interpolation::{FrameInterpolate, FrameInterpolationPlugin};
use shared::game::Wall;
use shared::protocol::physics::PLAYER_SIZE;
use shared::protocol::*;

#[cfg(feature = "debug")]
use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    prelude::*,
    text::FontSmoothing,
};

#[derive(Clone)]
pub struct ExampleRendererPlugin;

impl Plugin for ExampleRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);

        // draw after interpolation is done
        app.add_systems(PostStartup, draw_walls_retained);
        app.add_systems(
            PostUpdate,
            (draw_players, draw_circles)
                .after(InterpolationSystems::Interpolate)
                .after(RollbackSystems::VisualCorrection),
        );
        app.add_systems(Update, camera_follow);

        // add visual interpolation for Position and Rotation
        // (normally we would interpolate on Transform but here this is fine
        // since rendering is done via Gizmos that only depend on Position/Rotation)
        app.add_plugins(FrameInterpolationPlugin::<Position>::default());
        app.add_plugins(FrameInterpolationPlugin::<Rotation>::default());
        app.add_observer(add_frame_interpolation_components);

        #[cfg(feature = "debug")]
        {
            app.add_plugins(FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 42.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::srgb(0.0, 1.0, 0.0),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: true,
                        // The minimum acceptable fps
                        min_fps: 30.0,
                        // The target fps
                        target_fps: 144.0,
                    },
                },
            });
        }
    }
}

#[derive(Component)]
pub struct GameplayCamera;

fn init(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::from(OrthographicProjection {
            scale: 1.0,
            ..OrthographicProjection::default_2d()
        }),
        GameplayCamera,
    ));
}

fn camera_follow(
    mut cam: Query<&mut Transform, (With<GameplayCamera>, Without<Controlled>)>,
    to_follow: Query<&Transform, (Without<GameplayCamera>, With<Controlled>)>,
) {
    let Ok(mut cam) = cam.single_mut() else {
        return;
    };
    let Ok(tf) = to_follow.single() else {
        return;
    };
    cam.translation = tf.translation;
}

/// Predicted entities get updated in FixedUpdate, so we want to smooth/interpolate
/// their components in PostUpdate
fn add_frame_interpolation_components(
    // We use Position because it's added by avian later, and when it's added
    // we know that Predicted is already present on the entity
    trigger: On<Add, Position>,
    query: Query<Entity, With<Predicted>>,
    mut commands: Commands,
) {
    if query.contains(trigger.entity) {
        commands.entity(trigger.entity).insert((
            FrameInterpolate::<Position>::default(),
            FrameInterpolate::<Rotation>::default(),
        ));
    }
}

pub(crate) fn draw_players(
    mut gizmos: Gizmos,
    players: Query<(&Position, &Rotation, &ColorComponent), With<PlayerId>>,
) {
    for (position, rotation, color) in &players {
        gizmos.circle_2d(
            Isometry2d {
                rotation: Rot2 {
                    sin: rotation.sin,
                    cos: rotation.cos,
                },
                translation: Vec2::new(position.x, position.y),
            },
            PLAYER_SIZE,
            color.0,
        );
    }
}

pub(crate) fn draw_walls_retained(
    mut commands: Commands,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
    walls: Query<(&Wall, &ColorComponent), Without<PlayerId>>,
) {
    for (wall, color) in &walls {
        let mut gizmo = GizmoAsset::default();

        gizmo.line_2d(wall.start, wall.end, color.0);
        commands.spawn(Gizmo {
            handle: gizmo_assets.add(gizmo),
            line_config: GizmoLineConfig {
                width: 1.,
                ..default()
            },
            ..default()
        });
    }
}

/// System that draws circles
pub(crate) fn draw_circles(mut gizmos: Gizmos, circles: Query<&Position, With<CircleMarker>>) {
    for position in &circles {
        gizmos.circle_2d(Isometry2d::from_translation(position.0), 10.0, GREEN);
    }
}
