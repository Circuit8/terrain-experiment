use bevy;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::AmbientLight,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{shape, Mesh, VertexAttributeValues},
        pipeline::{PipelineDescriptor, PrimitiveTopology, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};
use bevy_flycam::PlayerPlugin;
use color_eyre::Report;
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Perlin, Seedable,
};
use rand::Rng;

const MAP_WIDTH: usize = 512;
const MAP_HEIGHT: f64 = 40.0;
const SUN_HEIGHT: f64 = MAP_HEIGHT + 50.0;

fn main() -> Result<(), Report> {
    init()?;

    App::build()
        .insert_resource(WindowDescriptor {
            title: "Josh's World".to_string(),
            width: 2000.,
            height: 1200.,
            vsync: false,
            ..Default::default()
        })
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_asset::<WaterMaterial>()
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

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "3bf9e364-f29d-4d6c-92cf-93298466c621"]
struct WaterMaterial {
    pub color: Color,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut pbr_materials: ResMut<Assets<StandardMaterial>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    asset_server: ResMut<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut vertices: Vec<[f32; 3]> = vec![];

    let perlin = Perlin::new();
    let seed = rand::thread_rng().gen_range(0..u32::MAX);
    perlin.set_seed(seed);
    let builder = PlaneMapBuilder::new(&perlin).set_size(MAP_WIDTH, MAP_WIDTH);
    let noise_map = builder.build();

    for x in 0..MAP_WIDTH {
        for z in 0..MAP_WIDTH {
            let top_left = [
                x as f32,
                (noise_map.get_value(x, z) * MAP_HEIGHT) as f32,
                z as f32,
            ];
            let bottom_left = [
                x as f32,
                (noise_map.get_value(x, z + 1) * MAP_HEIGHT) as f32,
                (z + 1) as f32,
            ];
            let bottom_right = [
                (x + 1) as f32,
                (noise_map.get_value(x + 1, z + 1) * MAP_HEIGHT) as f32,
                (z + 1) as f32,
            ];
            let top_right = [
                (x + 1) as f32,
                (noise_map.get_value(x + 1, z) * MAP_HEIGHT) as f32,
                z as f32,
            ];

            // Triangle: ◺
            vertices.push(bottom_right);
            vertices.push(top_left);
            vertices.push(bottom_left);
            // Triangle: ◹
            vertices.push(top_right);
            vertices.push(top_left);
            vertices.push(bottom_right);
        }
    }

    let uvs = vec![[0.0, 0.0, 0.0]; vertices.len()];
    // might have to do something different with the normals when we have heights
    let normals = vec![[0.0f32, 1.0f32, 0.0f32]; vertices.len()];

    mesh.set_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float3(vertices),
    );
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, VertexAttributeValues::Float3(uvs));
    mesh.set_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float3(normals),
    );

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        material: pbr_materials.add(StandardMaterial {
            base_color: Color::rgb_u8(123, 180, 78),
            roughness: 1.0,
            reflectance: 0.2,
            ..Default::default()
        }),
        ..Default::default()
    });

    // Watch for changes
    asset_server.watch_for_changes().unwrap();

    // Create a new shader pipeline with shaders loaded from the asset directory
    let pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shaders/water.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shaders/water.frag")),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind WaterMaterial resources to
    // our shader
    render_graph.add_system_node(
        "water_material",
        AssetRenderResourcesNode::<WaterMaterial>::new(true),
    );

    // Add a Render Graph edge connecting our new "water_material" node to the main pass node. This
    // ensures "water_material" runs before the main pass
    render_graph
        .add_node_edge("water_material", base::node::MAIN_PASS)
        .unwrap();

    let water_material = water_materials.add(WaterMaterial {
        color: Color::rgb(0.01, 0.2, 0.8),
    });

    // water surface
    let horizontal_plane_transform = MAP_WIDTH as f32 / 2.0 - 0.5;
    commands
        .spawn_bundle(MeshBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: MAP_WIDTH as f32,
            })),
            render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                pipeline_handle.clone(),
            )]),
            transform: Transform::from_xyz(
                horizontal_plane_transform,
                -0.5,
                horizontal_plane_transform,
            ),
            ..Default::default()
        })
        .insert(water_material.clone());

    // The sun
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere {
            radius: 10.0,
            subdivisions: 10,
        })),
        material: pbr_materials.add(Color::rgb(1.0, 0.9, 0.1).into()),
        transform: Transform::from_xyz(
            MAP_WIDTH as f32 / 2.0,
            SUN_HEIGHT as f32,
            MAP_WIDTH as f32 / 2.0,
        ),
        ..Default::default()
    });

    commands.spawn_bundle(LightBundle {
        light: Light {
            color: Color::rgb(1.0, 0.9, 0.1),
            intensity: 4000.0,
            fov: f32::to_radians(170.0),
            range: (MAP_WIDTH as f32).max(SUN_HEIGHT as f32),
            ..Default::default()
        },
        transform: Transform::from_xyz(
            MAP_WIDTH as f32 / 2.0,
            SUN_HEIGHT as f32,
            MAP_WIDTH as f32 / 2.0,
        ),
        ..Default::default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });
}
