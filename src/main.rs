use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
use bevy_frustum_culling::*;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Perlin, Seedable,
};
use rand::Rng;

const MAP_WIDTH: usize = 8;
const MAP_HEIGHT: f64 = 10.0;
const SUN_HEIGHT: f64 = MAP_HEIGHT + 5.0;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(BoundingVolumePlugin::<obb::Obb>::default())
        .add_plugin(FrustumCullingPlugin::<obb::Obb>::default())
        .add_plugin(PlayerPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_startup_system(setup.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // cubes
    let perlin = Perlin::new();
    perlin.set_seed(rand::thread_rng().gen_range(0..u32::MAX));
    let builder = PlaneMapBuilder::new(&perlin).set_size(MAP_WIDTH, MAP_WIDTH);
    let noise_map = builder.build();
    for z in (0..MAP_WIDTH as usize).into_iter() {
        for x in (0..MAP_WIDTH as usize).into_iter() {
            let noise_value = noise_map.get_value(x, z);
            let height = (noise_value * 10.0).floor() as i64 + 1;

            for y in (-6..height).into_iter() {
                let color = match height {
                    -5 => Color::rgb(0.3, 0.3, 0.3),
                    -4..=0 => Color::rgb(0.2, 0.2, 0.8),
                    _ => Color::rgb(0.2, 0.8, 0.2),
                };

                commands.spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                    material: materials.add(color.into()),
                    transform: Transform::from_xyz(x as f32, y as f32, z as f32),
                    ..Default::default()
                });
            }
        }
    }

    // water surface
    let horizontal_plane_transform = MAP_WIDTH as f32 / 2.0 - 0.5;
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: MAP_WIDTH as f32,
        })),
        material: materials.add(Color::rgba(0.1, 0.1, 0.95, 0.5).into()),
        transform: Transform::from_xyz(
            horizontal_plane_transform,
            -0.5,
            horizontal_plane_transform,
        ),
        ..Default::default()
    });

    // Floor
    let horizontal_plane_transform = MAP_WIDTH as f32 / 2.0 - 0.5;
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: MAP_WIDTH as f32,
        })),
        material: materials.add(Color::rgba(0.1, 0.1, 0.1, 1.0).into()),
        transform: Transform::from_xyz(
            horizontal_plane_transform,
            -5.5,
            horizontal_plane_transform,
        ),
        ..Default::default()
    });

    // light
    commands.spawn_bundle(LightBundle {
        light: Light {
            color: Color::rgb(1.0, 0.3, 0.9),
            intensity: 1000.0,
            fov: f32::to_radians(360.0),
            ..Default::default()
        },
        transform: Transform::from_xyz(4.0, SUN_HEIGHT as f32, 4.0),
        ..Default::default()
    });
    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(FrustumCulling);
}
