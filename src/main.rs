use bevy;
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    reflect::TypeUuid,
    render::renderer::RenderResources,
};
use bevy_flycam::{MovementSettings, PlayerPlugin};
use bevy_inspector_egui::InspectorPlugin;
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
        .add_plugins(DefaultPlugins)
        .add_plugin(PlayerPlugin)
        .add_plugin(InspectorPlugin::<terrain::Config>::new())
        .insert_resource(MovementSettings {
            sensitivity: 0.00010, // default: 0.00012
            speed: 40.0,          // default: 12.0
        })
        .insert_resource(ClearColor(Color::rgb_u8(142, 192, 255)))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_asset::<terrain::WaterMaterial>()
        .add_startup_system(setup.system())
        .add_startup_system(terrain::setup.system())
        .add_system(terrain::rebuild_on_change.system())
        .add_system(increase_shaders_time.system())
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
#[uuid = "463e4b8a-d555-4fc2-ba9f-4c880063ba92"]
struct TimeUniform {
    value: f32,
}

fn setup() {
    println!("General setup");
}

/// In this system we query for the `TimeComponent` and global `Time` resource, and set
/// `time.seconds_since_startup()` as the `value` of the `TimeComponent`. This value will be
/// accessed by the fragment shader and used to animate the shader.
fn increase_shaders_time(time: Res<Time>, mut query: Query<&mut TimeUniform>) {
    for mut time_uniform in query.iter_mut() {
        time_uniform.value = time.seconds_since_startup() as f32;
    }
}
