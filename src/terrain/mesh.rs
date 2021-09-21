use bevy::{
    math::Vec3,
    render::{
        mesh::{Indices, Mesh, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
};
use noise::utils::NoiseMap;

pub struct Generator {
    pub height_map: NoiseMap,
    pub height_scale: f64,
    pub simplification_level: u32,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    triangles_index: u32,
}

impl Generator {
    pub fn new(height_map: NoiseMap, height_scale: f64, simplification_level: u32) -> Generator {
        Generator {
            height_map,
            height_scale,
            simplification_level,
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

        let mesh_simplification_increment = if self.simplification_level == 0 {
            1
        } else {
            (self.simplification_level * 2) as usize
        };
        let vertices_per_line = (map_width - 1) / mesh_simplification_increment + 1;

        let mut vertex_index = 0;
        let mut y = 0;
        while y < map_height {
            let mut x = 0;
            while x < map_width {
                let height = self.height_map.get_value(x, y) * self.height_scale;

                self.vertices[vertex_index] = [x as f32, height as f32, y as f32];
                self.uvs[vertex_index] =
                    [x as f32 / map_width as f32, y as f32 / map_height as f32];

                if x < map_width - 1 && y < map_height - 1 {
                    let top_left = vertex_index;
                    let top_right = vertex_index + 1;
                    let bottom_left = vertex_index + vertices_per_line;
                    let bottom_right = vertex_index + vertices_per_line + 1;
                    self.add_triangle(bottom_right, top_left, bottom_left);
                    self.add_triangle(top_left, bottom_right, top_right);
                }

                vertex_index += 1;
                x += mesh_simplification_increment;
            }
            y += mesh_simplification_increment;
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
