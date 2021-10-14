use bevy::{
    core::FixedTimestep,
    diagnostic::{EntityCountDiagnosticsPlugin, FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log::info,
    prelude::*,
    reflect::TypeUuid,
    render::{renderer::RenderResources, wireframe::WireframePlugin},
    wgpu::{WgpuFeature, WgpuFeatures, WgpuOptions},
};
use bevy_inspector_egui::{widgets::ResourceInspector, Inspectable, InspectorPlugin};
use bevy_rapier3d::{
    physics::{
        ColliderBundle, ColliderPositionSync, NoUserData, RapierPhysicsPlugin, RigidBodyBundle,
    },
    prelude::ColliderShape,
    render::{ColliderDebugRender, RapierRenderPlugin},
};
use color_eyre::Report;

use crate::first_person::PlayerPlugin;
use crate::terrain::Terrain;

mod first_person;
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
        .insert_resource(WgpuOptions {
            features: WgpuFeatures {
                features: vec![WgpuFeature::NonFillPolygonMode], // Wireframe rendering for debugging requires NonFillPolygonMode feature
            },
            ..Default::default()
        })
        // .add_plugin(NoCameraPlayerPlugin)
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<Config>::new())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(EntityCountDiagnosticsPlugin::default())
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugin(WgpuResourceDiagnosticsPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(Terrain)
        .add_plugin(PlayerPlugin)
        .add_plugin(WireframePlugin)
        .add_startup_system(setup.system())
        .add_system(increase_shaders_time.system())
        .add_stage_after(
            CoreStage::Update,
            SlowUpdateStage,
            SystemStage::parallel()
                .with_run_criteria(FixedTimestep::step(2.0))
                .with_system(debug_player_position.system()),
        )
        .add_plugin(RapierRenderPlugin)
        .add_startup_system(test.system())
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
}

fn setup(mut commands: Commands) {
    commands.insert_resource(ClearColor(Color::rgb_u8(190, 246, 255)));
}

/// In this system we query for the `TimeComponent` and global `Time` resource, and set
/// `time.seconds_since_startup()` as the `value` of the `TimeComponent`. This value will be
/// accessed by the fragment shader and used to animate the shader.
fn increase_shaders_time(time: Res<Time>, mut query: Query<&mut TimeUniform>) {
    for mut time_uniform in query.iter_mut() {
        time_uniform.value = time.seconds_since_startup() as f32;
    }
}

fn debug_player_position(query: Query<&Transform, With<Player>>) {
    for transform in query.iter() {
        info!("Player position: {:?}", transform.translation);
    }
}

fn test(mut commands: Commands) {
    let y = 150.0;
    let mut color = 0;
    let rad = 0.5;

    for x in -24..24 {
        for z in -24..24 {
            color += 1;

            // Build the rigid body.
            let rigid_body = RigidBodyBundle {
                position: [(x * 10) as f32, y, (z * 10) as f32].into(),
                ..RigidBodyBundle::default()
            };

            let collider = ColliderBundle {
                shape: ColliderShape::cuboid(rad, rad, rad),
                ..ColliderBundle::default()
            };

            commands
                .spawn()
                .insert_bundle(rigid_body)
                .insert_bundle(collider)
                .insert(ColliderDebugRender::with_id(color))
                .insert(ColliderPositionSync::Discrete);
        }
    }
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "463e4b8a-d555-4fc2-ba9f-4c880063ba92"]
struct TimeUniform {
    value: f32,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct SlowUpdateStage;

pub struct Player;
