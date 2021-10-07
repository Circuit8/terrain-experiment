use bevy::{
    app::{Events, ManualEventReader},
    input::mouse::MouseMotion,
    prelude::*,
    render::camera::PerspectiveProjection,
};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_rapier3d::{
    physics::{ColliderBundle, ColliderPositionSync, RigidBodyBundle},
    prelude::{ColliderShape, PhysicsPipeline},
    render::ColliderDebugRender,
};

use crate::Player;

mod mouse;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<InputState>()
            .add_plugin(InspectorPlugin::<MovementSettings>::new())
            .add_startup_system(setup_player.system())
            .add_startup_system(mouse::initial_grab.system())
            .add_system(player_move.system())
            .add_system(player_look.system())
            .add_system(mouse::grab.system())
            .add_startup_system(enable_physics_profiling.system());
    }
}

fn enable_physics_profiling(mut pipeline: ResMut<PhysicsPipeline>) {
    pipeline.counters.enable()
}

fn setup_player(mut commands: Commands) {
    let player_radius = 1.0;
    let start_height = 60.0;
    let transform = Transform::from_xyz(0.0, start_height, 0.0).looking_at(Vec3::ZERO, Vec3::Y);

    let rigid_body = RigidBodyBundle {
        position: [0.0, start_height, 0.0].into(),
        ..RigidBodyBundle::default()
    };

    let collider = ColliderBundle {
        shape: ColliderShape::cuboid(player_radius, player_radius, player_radius),
        ..ColliderBundle::default()
    };

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform,
            perspective_projection: PerspectiveProjection {
                far: 5000.0,
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(rigid_body)
        .insert(collider)
        .insert(ColliderPositionSync::Discrete)
        .insert(ColliderDebugRender::with_id(0))
        .insert(Player);
}

/// Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    time: Res<Time>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    for (_camera, mut transform) in query.iter_mut() {
        let mut velocity = Vec3::ZERO;
        let local_z = transform.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);

        for key in keys.get_pressed() {
            if window.cursor_locked() {
                if validate_key(settings.map.forward, key) {
                    velocity += forward
                }
                if validate_key(settings.map.backward, key) {
                    velocity -= forward
                }
                if validate_key(settings.map.left, key) {
                    velocity -= right
                }
                if validate_key(settings.map.right, key) {
                    velocity += right
                }
                if validate_key(settings.map.up, key) {
                    velocity += Vec3::Y
                }
                if validate_key(settings.map.down, key) {
                    velocity -= Vec3::Y
                }
            }
        }

        velocity = velocity.normalize();

        if !velocity.is_nan() {
            transform.translation += velocity * time.delta_seconds() * settings.speed
        }
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    for (_camera, mut transform) in query.iter_mut() {
        for ev in state.reader_motion.iter(&motion) {
            let sensitivity = settings.sensitivity / 10000.0; // to keep config in reasonable range
            if window.cursor_locked() {
                state.pitch -= (sensitivity * ev.delta.y * window.height()).to_radians();
                state.yaw -= (sensitivity * ev.delta.x * window.width()).to_radians();
            }

            state.pitch = state.pitch.clamp(-1.54, 1.54);

            // Order is important to prevent unintended roll
            transform.rotation = Quat::from_axis_angle(Vec3::Y, state.yaw)
                * Quat::from_axis_angle(Vec3::X, state.pitch);
        }
    }
}

fn validate_key<T>(codes: &'static [T], key: &T) -> bool
where
    T: PartialEq<T>,
{
    codes.iter().any(|m| m == key)
}

#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

#[derive(Inspectable)]
pub struct MovementSettings {
    #[inspectable(min = 0.1, max = 10.0)]
    pub sensitivity: f32,
    pub speed: f32,
    #[inspectable(ignore)]
    pub map: CamKeyMap,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 1.2,
            speed: 90.,
            map: CamKeyMap::default(),
        }
    }
}

pub struct CamKeyMap {
    pub forward: &'static [KeyCode],
    pub backward: &'static [KeyCode],
    pub left: &'static [KeyCode],
    pub right: &'static [KeyCode],
    pub jump: &'static [KeyCode],
    pub up: &'static [KeyCode],
    pub down: &'static [KeyCode],
}

impl Default for CamKeyMap {
    fn default() -> Self {
        Self {
            forward: &[KeyCode::W],
            backward: &[KeyCode::S],
            left: &[KeyCode::A],
            right: &[KeyCode::D],
            jump: &[KeyCode::Space],
            up: &[KeyCode::Space],
            down: &[KeyCode::LShift],
        }
    }
}
