use bevy;
use bevy::render::{
    mesh::{Mesh, VertexAttributeValues},
    pipeline::PrimitiveTopology,
};
use noise::{
    utils::{NoiseMapBuilder, PlaneMapBuilder},
    Perlin,
};

pub fn generate_mesh(
    map_width: usize,
    height_scale: f64, // increase to make hills higher
    noise_scale: f64,  // increase for more hills closer together
) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    let mut vertices: Vec<[f32; 3]> = vec![];

    let perlin = Perlin::new();
    let builder = PlaneMapBuilder::new(&perlin)
        .set_size(map_width, map_width)
        .set_x_bounds(-1.0 * noise_scale, 1.0 * noise_scale)
        .set_y_bounds(-1.0 * noise_scale, 1.0 * noise_scale);
    let noise_map = builder.build();

    noise_map.write_to_file("terrain-noise-map.png");

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
