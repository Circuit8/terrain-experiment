use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    pbr::AmbientLight,
    prelude::*,
    reflect::TypeUuid,
    render::{renderer::RenderResources, wireframe::WireframePlugin},
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};
use bevy_flycam::{MovementSettings, PlayerPlugin};
use bevy_inspector_egui::{widgets::ResourceInspector, Inspectable, InspectorPlugin};
use color_eyre::Report;

mod terrain;

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
        .insert_resource(MovementSettings {
            sensitivity: 0.00010, // default: 0.00012
            speed: 40.0,          // default: 12.0
        })
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                features: vec![WgpuFeature::NonFillPolygonMode], // Wireframe rendering for debugging requires NonFillPolygonMode feature
            },
            ..Default::default()
        })
        .add_plugin(PlayerPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Config>::new())
        .add_plugin(InspectorPlugin::<terrain::Config>::new())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(WireframePlugin)
        .add_startup_system(setup.system())
        .add_startup_system(terrain::endless::setup.system())
        .add_system(increase_shaders_time.system())
        .add_system(
            terrain::endless::initialize_chunks
                .system()
                .before("terrain::endless::compute_chunk_visibility"),
        )
        .add_system(
            terrain::endless::process_chunks
                .system()
                .before("terrain::endless::compute_chunk_visibility"),
        )
        .add_system(
            terrain::endless::insert_chunks
                .system()
                .before("terrain::endless::compute_chunk_visibility"),
        )
        .add_system(
            terrain::endless::compute_chunk_visibility
                .system()
                .label("terrain::endless::compute_chunk_visibility"),
        )
        .add_system(
            terrain::endless::rebuild_on_change
                .system()
                .after("terrain::endless::compute_chunk_visibility"),
        )
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

#[derive(Inspectable, Default)]
struct Config {
    clear_color: ResourceInspector<ClearColor>,
    ambient_light: ResourceInspector<AmbientLight>,
}

fn setup(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::rgb_u8(197, 227, 241)));
    commands.insert_resource(AmbientLight {
        color: Color::rgb_u8(255, 253, 246),
        brightness: 0.28,
    });
    // Apparently ambient light editing doesnt work without a LightBundle?
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(0.0, 400.0, 0.0),
        ..Default::default()
    });
}

/// In this system we query for the `TimeComponent` and global `Time` resource, and set
/// `time.seconds_since_startup()` as the `value` of the `TimeComponent`. This value will be
/// accessed by the fragment shader and used to animate the shader.
fn increase_shaders_time(time: Res<Time>, mut query: Query<&mut TimeUniform>) {
    for mut time_uniform in query.iter_mut() {
        time_uniform.value = time.seconds_since_startup() as f32;
    }
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "463e4b8a-d555-4fc2-ba9f-4c880063ba92"]
struct TimeUniform {
    value: f32,
}
