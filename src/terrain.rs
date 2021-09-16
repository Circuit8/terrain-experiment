use bevy;
use bevy::{
    ecs::system::{Res, ResMut},
    math::Vec3,
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{shape, Indices, Mesh, VertexAttributeValues},
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
use std::convert::From;

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
    mut materials: ResMut<Assets<StandardMaterial>>,
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

        let mut terrain_mesh_generator = TerrainMeshGenerator::new(noise_map, config.height_scale);
        let terrain_mesh = terrain_mesh_generator.generate();

        commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(terrain_mesh),
                material: materials.add(StandardMaterial {
                    base_color: Color::GRAY,
                    ..Default::default()
                }),
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

struct TerrainMeshGenerator {
    pub height_map: NoiseMap,
    pub height_scale: f64,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    triangles_index: u32,
}

impl TerrainMeshGenerator {
    pub fn new(height_map: NoiseMap, height_scale: f64) -> TerrainMeshGenerator {
        TerrainMeshGenerator {
            height_map,
            height_scale,
            vertices: vec![],
            triangles: vec![],
            uvs: vec![],
            normals: vec![],
            triangles_index: 0,
        }
    }

    pub fn generate(&mut self) -> Mesh {
        let map_width = self.height_map.size().0;
        let map_height = self.height_map.size().1;
        let map_size = map_width * map_height;

        self.vertices = vec![[0., 0., 0.]; map_size];
        self.normals = vec![[0., 0., 0.]; map_size];
        self.uvs = vec![[0., 0.]; map_size];
        self.triangles = vec![0; (map_width - 1) * (map_height - 1) * 6];
        self.triangles_index = 0;

        let mut vertex_index = 0;
        for x in 0..map_width {
            for z in 0..map_height {
                let height = self.height_map.get_value(x, z) * self.height_scale;
                self.vertices[vertex_index] = [x as f32, height as f32, z as f32];
                self.uvs[vertex_index] =
                    [x as f32 / map_width as f32, z as f32 / map_height as f32];

                if x < map_width - 1 && z < map_height - 1 {
                    let top_left = vertex_index;
                    let top_right = vertex_index + 1;
                    let bottom_left = vertex_index + map_width;
                    let bottom_right = vertex_index + map_width + 1;
                    self.add_triangle(top_left, bottom_right, bottom_left);
                    self.add_triangle(bottom_right, top_left, top_right);
                }

                vertex_index += 1;
            }
        }

        self.create_mesh()
    }

    fn add_triangle(&mut self, a: usize, b: usize, c: usize) {
        self.triangles[self.triangles_index as usize] = a as u32;
        self.triangles[(self.triangles_index + 1) as usize] = b as u32;
        self.triangles[(self.triangles_index + 2) as usize] = c as u32;
        self.triangles_index += 3;
    }

    fn create_mesh(&mut self) -> Mesh {
        self.calculate_normals();

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(self.triangles.clone())));
        mesh.set_attribute(
            Mesh::ATTRIBUTE_POSITION,
            VertexAttributeValues::Float3(self.vertices.clone()),
        );
        mesh.set_attribute(
            Mesh::ATTRIBUTE_UV_0,
            VertexAttributeValues::Float2(self.uvs.clone()),
        );
        mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, self.normals.clone());

        return mesh;
    }

    // Right now this is not a perfect way of handling the normals.
    // What we should be doing is calculating the normal of each vertex, based on the average normal of each vertexes triangles its a part of
    // Instead were just setting the normal of all the vertexes of a triangle to the normal of that plane, and then overwriting some as we go along.
    // This will not give us the most realistic pbr lighting.
    fn calculate_normals(&mut self) {
        for triangle_indexes in self.triangles.chunks_exact(3) {
            let normal = self.face_normal(
                self.vertices[triangle_indexes[0] as usize],
                self.vertices[triangle_indexes[1] as usize],
                self.vertices[triangle_indexes[2] as usize],
            );

            self.normals[triangle_indexes[0] as usize] = normal;
            self.normals[triangle_indexes[1] as usize] = normal;
            self.normals[triangle_indexes[2] as usize] = normal;
        }
    }

    fn face_normal(&self, a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
        let (a, b, c) = (Vec3::from(a), Vec3::from(b), Vec3::from(c));
        (b - a).cross(c - a).into()
    }
}
