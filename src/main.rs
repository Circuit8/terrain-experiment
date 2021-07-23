use bevy::prelude::*;
use bevy_flycam::PlayerPlugin;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Perlin,
};

const MAP_WIDTH: usize = 32;
const MAP_HEIGHT: f64 = 10.0;
const SUN_HEIGHT: f64 = MAP_HEIGHT + 5.0;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup.system())
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let perlin = Perlin::new();
    let builder = PlaneMapBuilder::new(&perlin).set_size(MAP_WIDTH, MAP_WIDTH);
    let noise_map = builder.build();

    for z in (0..MAP_WIDTH as usize).into_iter() {
        for x in (0..MAP_WIDTH as usize).into_iter() {
            let noise_value = noise_map.get_value(x, z);
            let height = (noise_value * 10.0).floor() as i64 + 1;
            print!("{} ", height);

            for y in (-5..height).into_iter() {
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
        println!("");
    }

    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, SUN_HEIGHT as f32, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
