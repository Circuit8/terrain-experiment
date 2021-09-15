use bevy;
use bevy::{
    ecs::system::{Res, ResMut},
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{shape, Mesh, VertexAttributeValues},
        pipeline::PrimitiveTopology,
        pipeline::{PipelineDescriptor, RenderPipeline},
        render_graph::{base, AssetRenderResourcesNode, RenderGraph, RenderResourcesNode},
        renderer::RenderResources,
        shader::ShaderStages,
    },
};
use bevy_inspector_egui::Inspectable;
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Seedable,
};

use crate::TimeUniform;

#[derive(Inspectable)]
pub struct Config {
    #[inspectable(min = 2)]
    map_width: usize,
    #[inspectable(min = 0.0001)]
    noise_scale: f64,
    #[inspectable(min = 1)]
    seed: u32,
    #[inspectable(min = 0.0001)]
    lacunarity: f64, // increase for more hills closer together
    #[inspectable(min = 0.0001)]
    persistance: f64,
    #[inspectable(min = 1)]
    octaves: usize,
    #[inspectable(min = 1.0)]
    height_scale: f64,
    #[inspectable(min = 0.0)]
    sun_height: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            map_width: 512,
            height_scale: 60.0,
            noise_scale: 1.8,
            seed: 2,
            octaves: 4,
            lacunarity: 2.0,
            persistance: 0.5,
            sun_height: 80.0,
        }
    }
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "3bf9e364-f29d-4d6c-92cf-93298466c621"]
pub struct WaterMaterial {
    pub color: Color,
}

#[derive(RenderResources, Default, TypeUuid)]
#[uuid = "93fb26fc-6c05-489b-9029-601edf703b6b"]
struct GrassTexture {
    pub texture: Handle<Texture>,
}

pub struct Terrain;

pub struct TerrainAssetHandles {
    pub water_material: Handle<WaterMaterial>,
    pub terrain_pipeline: Handle<PipelineDescriptor>,
    pub water_pipeline: Handle<PipelineDescriptor>,
    pub sun_pipeline: Handle<PipelineDescriptor>,
}

pub fn setup(
    mut pipelines: ResMut<Assets<PipelineDescriptor>>,
    asset_server: ResMut<AssetServer>,
    mut render_graph: ResMut<RenderGraph>,
    mut water_materials: ResMut<Assets<WaterMaterial>>,
    mut commands: Commands,
) {
    // Terrain Shader
    let terrain_pipeline = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shaders/terrain.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shaders/terrain.frag")),
    }));

    // Create a new shader pipeline with shaders loaded from the asset directory
    let water_pipeline = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
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

    let sun_pipeline = pipelines.add(PipelineDescriptor::default_config(ShaderStages {
        vertex: asset_server.load::<Shader, _>("shaders/sun.vert"),
        fragment: Some(asset_server.load::<Shader, _>("shaders/sun.frag")),
    }));

    commands.insert_resource(TerrainAssetHandles {
        water_material,
        terrain_pipeline,
        water_pipeline,
        sun_pipeline,
    })
}

// Rebuild the terrain if it changes
pub fn rebuild_on_change(
    mut commands: Commands,
    config: Res<Config>,
    asset_handles: Res<TerrainAssetHandles>,
    mut meshes: ResMut<Assets<Mesh>>,
    terrain_query: Query<(Entity, &Terrain)>,
) {
    if config.is_changed() {
        // Destroy all the previous terrain entities like the water, ground, sun etc (we'll recreate them all)
        for (entity, _) in terrain_query.iter() {
            commands.entity(entity).despawn()
        }

        let noise_map = generate_noise_map(
            config.map_width,
            config.noise_scale,
            config.seed,
            config.lacunarity,
            config.persistance,
            config.octaves,
        );

        let terrain_mesh = generate_mesh(&noise_map, config.map_width, config.height_scale);

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(terrain_mesh),
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    asset_handles.terrain_pipeline.clone(),
                )]),
                ..Default::default()
            })
            .insert(Terrain);

        // water surface
        let horizontal_plane_transform = config.map_width as f32 / 2.0 - 0.5;
        commands
            .spawn_bundle(MeshBundle {
                mesh: meshes.add(Mesh::from(shape::Plane {
                    size: config.map_width as f32,
                })),
                render_pipelines: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                    asset_handles.water_pipeline.clone(),
                )]),
                transform: Transform::from_xyz(
                    horizontal_plane_transform,
                    -0.5,
                    horizontal_plane_transform,
                ),
                ..Default::default()
            })
            .insert(asset_handles.water_material.clone())
            .insert(TimeUniform { value: 0.0 })
            .insert(Terrain);

        // The sun
        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Icosphere {
                    radius: 10.0,
                    subdivisions: 10,
                })),
                transform: Transform::from_xyz(
                    config.map_width as f32 / 2.0,
                    config.sun_height as f32, // pub const SUN_HEIGHT: f64 = MAP_HEIGHT_SCALE + 50.0;
                    config.map_width as f32 / 2.0,
                ),
                ..Default::default()
            })
            .insert(Terrain);
    }
}

pub fn generate_noise_map(
    map_width: usize,
    noise_scale: f64, // increase for more hills closer together
    seed: u32,
    lacunarity: f64,
    persistance: f64,
    octaves: usize,
) -> NoiseMap {
    let fbm = Fbm::new()
        .set_seed(seed)
        .set_lacunarity(lacunarity)
        .set_persistence(persistance)
        .set_octaves(octaves);
    let builder = PlaneMapBuilder::new(&fbm)
        .set_size(map_width, map_width)
        .set_x_bounds(-1.0 * noise_scale, 1.0 * noise_scale)
        .set_y_bounds(-1.0 * noise_scale, 1.0 * noise_scale);
    builder.build()
}

pub fn generate_mesh(noise_map: &NoiseMap, map_width: usize, height_scale: f64) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut vertices: Vec<[f32; 3]> = vec![];

    for x in 0..map_width {
        for z in 0..map_width {
            let top_left = [
                x as f32,
                (noise_map.get_value(x, z) * height_scale) as f32,
                z as f32,
            ];
            let bottom_left = [
                x as f32,
                (noise_map.get_value(x, z + 1) * height_scale) as f32,
                (z + 1) as f32,
            ];
            let bottom_right = [
                (x + 1) as f32,
                (noise_map.get_value(x + 1, z + 1) * height_scale) as f32,
                (z + 1) as f32,
            ];
            let top_right = [
                (x + 1) as f32,
                (noise_map.get_value(x + 1, z) * height_scale) as f32,
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

    return mesh;
}
