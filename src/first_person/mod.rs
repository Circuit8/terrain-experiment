use bevy::{
    app::{Events, ManualEventReader},
    input::mouse::MouseMotion,
    prelude::*,
    render::camera::PerspectiveProjection,
};
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use bevy_rapier3d::{
    na::Vector,
    physics::{ColliderBundle, RapierConfiguration, RigidBodyBundle, RigidBodyPositionSync},
    prelude::{
        ColliderMassProps, ColliderShape, PhysicsPipeline, RigidBodyActivation, RigidBodyDamping,
        RigidBodyForces, RigidBodyMassProps, RigidBodyMassPropsFlags, RigidBodyType,
        RigidBodyVelocity,
    },
    render::RapierRenderPlugin,
};

use crate::Player;

mod mouse;

struct PlayerEyes;
struct EyesEntity(Entity);
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<MouseState>()
            .insert_resource(RapierConfiguration {
                gravity: Vector::y() * -50.0,
                ..Default::default()
            })
            .add_plugin(InspectorPlugin::<MovementConfig>::new())
            .add_plugin(RapierRenderPlugin)
            .add_startup_system(setup_player.system())
            .add_startup_system(mouse::initial_grab.system())
            .add_system(player_move.system())
            .add_system(player_look.system())
            .add_system(mouse::grab.system())
            .add_system(config_change.system())
            .add_startup_system(enable_physics_profiling.system());
    }
}

fn setup_player(mut commands: Commands) {
    let start_height = 200.0;
    let transform = Transform::from_xyz(20.0, start_height, 20.0).looking_at(Vec3::ZERO, Vec3::Y);

    let rigid_body = RigidBodyBundle {
        forces: RigidBodyForces {
            gravity_scale: 0.0,
            ..Default::default()
        },
        mass_properties: RigidBodyMassPropsFlags::ROTATION_LOCKED.into(),
        position: [0.0, start_height, 0.0].into(),
        damping: RigidBodyDamping {
            linear_damping: 0.0,
            angular_damping: 0.0,
        },
        activation: RigidBodyActivation {
            sleeping: false,
            ..Default::default()
        },
        body_type: RigidBodyType::Dynamic,
        ..RigidBodyBundle::default()
    };

    let collider = ColliderBundle {
        mass_properties: ColliderMassProps::Density(100.0),
        shape: ColliderShape::cuboid(0.5, 2.0, 0.5),
        ..ColliderBundle::default()
    };

    let player = commands
        .spawn()
        .insert_bundle(rigid_body)
        .insert_bundle(collider)
        .insert(RigidBodyPositionSync::Interpolated { prev_pos: None })
        .insert(transform)
        .insert(Player)
        .id();

    let eyes = commands
        .spawn_bundle(PerspectiveCameraBundle {
            perspective_projection: PerspectiveProjection {
                far: 5000.0,
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(0.0, 1.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PlayerEyes)
        .id();

    commands
        .entity(player)
        .insert(EyesEntity(eyes))
        .push_children(&[eyes]);
}

/// Handles keyboard input and movement
fn player_move(
    time: Res<Time>,
    keys: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    mut config: ResMut<MovementConfig>,
    mut query: Query<(
        &Player,
        &mut RigidBodyVelocity,
        &RigidBodyMassProps,
        &EyesEntity,
    )>,
    player_eyes_query: Query<(&PlayerEyes, &Transform)>,
) {
    let window = windows.get_primary().unwrap();
    for (_player, mut velocity, mass_props, eyes_entity) in query.iter_mut() {
        config.sim_to_render += time.delta_seconds();

        let looking = player_eyes_query
            .get_component::<Transform>(eyes_entity.0)
            .expect("Failed to get Transform from Eyes");

        let mut desired_direction = Vec3::ZERO;
        let local_z = looking.local_z();
        let forward = -Vec3::new(local_z.x, 0., local_z.z);
        let right = Vec3::new(local_z.z, 0., -local_z.x);

        for key in keys.get_pressed() {
            if window.cursor_locked() {
                if validate_key(config.map.forward, key) {
                    desired_direction += forward
                }
                if validate_key(config.map.backward, key) {
                    desired_direction -= forward
                }
                if validate_key(config.map.left, key) {
                    desired_direction -= right
                }
                if validate_key(config.map.right, key) {
                    desired_direction += right
                }
                if validate_key(config.map.up, key) {
                    desired_direction += Vec3::Y
                }
                if validate_key(config.map.down, key) {
                    desired_direction -= Vec3::Y
                }
            }
        }

        if config.sim_to_render < config.dt {
            continue;
        }
        // Calculate the remaining simulation to render time after all
        // simulation steps were taken
        config.sim_to_render %= config.dt;

        let current_velocity: Vec3 = velocity.linvel.into();
        let current_ground_velocity = current_velocity * Vec3::new(1.0, 0.0, 1.0);

        let desired_velocity = if desired_direction.length_squared() > 1E-6 {
            desired_direction.normalize() * config.speed
        } else {
            // No input, damp the velocity so we dont keep gliding off into the distance
            current_ground_velocity * 0.5
        };

        // Calculate impulse - the desired momentum change for the time period
        let delta_velocity = desired_velocity - current_ground_velocity;
        let impulse = delta_velocity * mass_props.mass();
        if impulse.length_squared() > 1E-6 {
            velocity.apply_impulse(mass_props, impulse.into());
        }
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    config: Res<MovementConfig>,
    windows: Res<Windows>,
    mut state: ResMut<MouseState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<(&PlayerEyes, &mut Transform)>,
) {
    let window = windows.get_primary().unwrap();
    for (_camera, mut transform) in query.iter_mut() {
        for ev in state.reader_motion.iter(&motion) {
            let sensitivity = config.sensitivity / 10000.0; // to keep config in reasonable range
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

fn config_change(
    config: Res<MovementConfig>,
    mut rapier_config: ResMut<RapierConfiguration>,
    mut player_query: Query<&mut RigidBodyForces, With<Player>>,
) {
    if config.is_changed() {
        let mut forces = player_query.iter_mut().next().unwrap();
        forces.gravity_scale = if config.gravity { 1.0 } else { 0.0 };

        rapier_config.gravity = Vector::y() * config.gravity_strength;
    }
}

fn enable_physics_profiling(mut pipeline: ResMut<PhysicsPipeline>) {
    pipeline.counters.enable()
}

fn validate_key<T>(codes: &'static [T], key: &T) -> bool
where
    T: PartialEq<T>,
{
    codes.iter().any(|m| m == key)
}

#[derive(Default)]
struct MouseState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

#[derive(Inspectable)]
pub struct MovementConfig {
    #[inspectable(min = 0.1, max = 10.0)]
    pub sensitivity: f32,
    pub speed: f32,
    dt: f32,
    gravity: bool,
    gravity_strength: f32,
    #[inspectable(ignore)]
    sim_to_render: f32,
    #[inspectable(ignore)]
    pub map: CamKeyMap,
}

impl Default for MovementConfig {
    fn default() -> Self {
        Self {
            sensitivity: 1.2,
            speed: 60.,
            dt: 1.0 / 60.0,
            gravity: true,
            gravity_strength: -50.0,
            sim_to_render: 0.0,
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
