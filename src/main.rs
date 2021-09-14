use bevy;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{shape, Mesh},
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};
use bevy_flycam::{MovementSettings, PlayerPlugin};
use color_eyre::Report;

mod settings;
mod terrain;

use crate::settings::{MAP_HEIGHT_SCALE, MAP_NOISE_SCALE, MAP_WIDTH, SUN_HEIGHT};

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
        .insert_resource(MovementSettings {
            sensitivity: 0.00010, // default: 0.00012
            speed: 40.0,          // default: 12.0
        })
        .insert_resource(ClearColor(Color::rgb_u8(142, 192, 255)))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_asset::<WaterMaterial>()
        .add_startup_system(setup.system())
        .add_system(animate_shader.system())
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

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "463e4b8a-d555-4fc2-ba9f-4c880063ba92"]
struct TimeUniform {
    value: f32,
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "93fb26fc-6c05-489b-9029-601edf703b6b"]
struct GrassTexture {
    pub texture: Handle<Texture>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    asset_server: ResMut<AssetServer>,
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    mut render_graph: ResMut<RenderGraph>,
) {
    // asset_server.load::<Texture, _>("textures/grass.jpg");

    let mesh = terrain::generate_mesh(MAP_WIDTH, MAP_HEIGHT_SCALE, MAP_NOISE_SCALE);

    // Terrain Shader
    let terrain_pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shaders/terrain.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shaders/terrain.frag")),
    }));

    // render_graph.add_system_node(
    //     "my_array_texture",
    //     AssetRenderResourcesNode::<GrassTexture>::new(true),
    // );
    // render_graph
    //     .add_node_edge("my_array_texture", base::node::MAIN_PASS)
    //     .unwrap();

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            terrain_pipeline_handle.clone(),
        )]),
        ..Default::default()
    });

    // Create a new shader pipeline with shaders loaded from the asset directory
    let water_pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shaders/mvp.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shaders/water.frag")),
    }));

    // Add an AssetRenderResourcesNode to our Render Graph. This will bind WaterMaterial resources to
    // our shader
    render_graph.add_system_node(
        "time_uniform",
        RenderResourcesNode::<TimeUniform>::new(true),
    );
    render_graph.add_system_node(
        "water_material",
        AssetRenderResourcesNode::<WaterMaterial>::new(true),
    );

    // Add a Render Graph edge connecting our new "water_material" node to the main pass node. This
    // ensures "water_material" runs before the main pass
    render_graph
        .add_node_edge("water_material", base::node::MAIN_PASS)
        .unwrap();
    render_graph
        .add_node_edge("time_uniform", base::node::MAIN_PASS)
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
                water_pipeline_handle.clone(),
            )]),
            transform: Transform::from_xyz(
                horizontal_plane_transform,
                -0.5,
                horizontal_plane_transform,
            ),
            ..Default::default()
        })
        .insert(water_material.clone())
        .insert(TimeUniform { value: 0.0 });

    // Sun Shader
    let sun_pipeline_handle = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shaders/sun.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shaders/sun.frag")),
    }));

    // The sun
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Icosphere {
            radius: 10.0,
            subdivisions: 10,
        })),
        render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
            sun_pipeline_handle.clone(),
        )]),
        transform: Transform::from_xyz(
            MAP_WIDTH as f32 / 2.0,
            SUN_HEIGHT as f32,
            MAP_WIDTH as f32 / 2.0,
        ),
        ..Default::default()
    });

    // only required with PBR right?
    // commands.spawn_bundle(LightBundle {
    //     light: Light {
    //         color: Color::rgb(1.0, 0.9, 0.1),
    //         intensity: 4000.0,
    //         fov: f32::to_radians(170.0),
    //         range: (MAP_WIDTH as f32).max(SUN_HEIGHT as f32),
    //         ..Default::default()
    //     },
    //     transform: Transform::from_xyz(
    //         MAP_WIDTH as f32 / 2.0,
    //         SUN_HEIGHT as f32,
    //         MAP_WIDTH as f32 / 2.0,
    //     ),
    //     ..Default::default()
    // });

    // commands.insert_resource(AmbientLight {
    //     color: Color::WHITE,
    //     brightness: 0.2,
    // });
}

/// In this system we query for the `TimeComponent` and global `Time` resource, and set
/// `time.seconds_since_startup()` as the `value` of the `TimeComponent`. This value will be
/// accessed by the fragment shader and used to animate the shader.
fn animate_shader(time: Res<Time>, mut query: Query<&mut TimeUniform>) {
    let mut time_uniform = query.single_mut().unwrap();
    time_uniform.value = time.seconds_since_startup() as f32;
}
