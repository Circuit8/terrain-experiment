use bevy::prelude::*;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    render::{
        mesh::{Indices, Mesh, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
};
use bevy_flycam::PlayerPlugin;
use bevy_frustum_culling::*;
use color_eyre::Report;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Perlin, Seedable,
};
use rand::Rng;

const MAP_WIDTH: usize = 24;
const MAP_HEIGHT: f64 = 10.0;
const SUN_HEIGHT: f64 = MAP_HEIGHT + 5.0;

fn main() -> Result<(), Report> {
    init()?;

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
    Ok(())
}

fn init() -> Result<(), Report> {
    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1")
    }
    color_eyre::install()?;

    Ok(())
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut shaders: ResMut<Assets<Shader>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut vertices = vec![[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 0.0]];
    let mut normals: Vec<[f32; 3]> = Vec::new();

    let pipeline_handle = add_terrain_material(pipelines, shaders, render_graph);

    normals.resize(3, [0.0f32, 1.0f32, 0.0f32]);
    let uvs = vec![[0.0, 0.0, 0.0]; vertices.len()];

    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float3(vertices),
    );
    // mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float3(uvs));
    mesh.set_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float3(normals),
    );

    commands.spawn_bundle(MeshBundle {
        mesh: meshes.add(mesh),
        transform: Transform::from_xyz(0.0 as f32, 0.0 as f32, 0.0 as f32),
        ..Default::default()
    });

    // // cubes
    // let perlin = Perlin::new();
    // let seed = rand::thread_rng().gen_range(0..u32::MAX);
    // perlin.set_seed(seed);
    // let builder = PlaneMapBuilder::new(&perlin).set_size(MAP_WIDTH, MAP_WIDTH);
    // let noise_map = builder.build();
    // for z in (0..MAP_WIDTH as usize).into_iter() {
    //     for x in (0..MAP_WIDTH as usize).into_iter() {
    //         let noise_value = noise_map.get_value(x, z);
    //         let height = (noise_value * 10.0).floor() as i64 + 1;

    //         for y in (-6..height).into_iter() {
    //             let color = match height {
    //                 -5 => Color::rgb(0.3, 0.3, 0.3),
    //                 -4..=0 => Color::rgb(0.2, 0.2, 0.8),
    //                 _ => Color::rgb(0.2, 0.8, 0.2),
    //             };

    //             commands.spawn_bundle(PbrBundle {
    // mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //                 material: materials.add(color.into()),
    //                 transform: Transform::from_xyz(x as f32, y as f32, z as f32),
    //                 ..Default::default()
    //             });
    //         }
    //     }
    // }

    // water surface
    // let horizontal_plane_transform = MAP_WIDTH as f32 / 2.0 - 0.5;
    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Plane {
    //         size: MAP_WIDTH as f32,
    //     })),
    //     material: materials.add(Color::rgba(0.1, 0.1, 0.95, 0.5).into()),
    //     transform: Transform::from_xyz(
    //         horizontal_plane_transform,
    //         -0.5,
    //         horizontal_plane_transform,
    //     ),
    //     ..Default::default()
    // });

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
