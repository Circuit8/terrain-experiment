use bevy;
use bevy::render::{
    mesh::{Mesh, VertexAttributeValues},
    pipeline::PrimitiveTopology,
};
use noise::{
    utils::{NoiseMap, NoiseMapBuilder, PlaneMapBuilder},
    Fbm, MultiFractal, Seedable,
};

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
