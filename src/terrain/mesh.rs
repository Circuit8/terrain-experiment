use bevy::{
    math::Vec3,
    render::{
        mesh::{Indices, Mesh, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
};
use bevy_rapier3d::{
    na::{DMatrix, Vector3},
    prelude::{ColliderShape, SharedShape},
};

use super::{height_map::HeightMap, SimplificationLevel};

pub struct Generator {
    pub height_map: HeightMap,
    pub height_scale: f32,
    pub simplification_level: SimplificationLevel,
    pub simplification_increment: usize,
    pub vertices_per_line: usize,
    pub vertices: Vec<[f32; 3]>,
    pub triangles: Vec<u32>,
    pub uvs: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    pub map_width: usize,
    triangles_index: u32,
}

impl Generator {
    pub fn new(
        height_map: HeightMap,
        height_scale: f32,
        simplification_level: SimplificationLevel,
    ) -> Generator {
        let map_width = height_map.data.len();

        let simplification_increment = if simplification_level == SimplificationLevel(0) {
            1
        } else {
            (simplification_level.0 * 2) as usize
        };
        let vertices_per_line = (map_width - 1) / simplification_increment + 1;

        Generator {
            height_map,
            height_scale,
            simplification_level,
            simplification_increment,
            vertices_per_line,
            map_width,
            vertices: vec![],
            triangles: vec![],
            uvs: vec![],
            normals: vec![],
            triangles_index: 0,
        }
    }

    pub fn generate(&mut self) {
        let map_size = self.map_width * self.map_width;

        self.vertices = vec![[0., 0., 0.]; map_size];
        self.normals = vec![[0., 0., 0.]; map_size];
        self.uvs = vec![[0., 0.]; map_size];
        self.triangles = vec![0; (self.map_width - 1) * (self.map_width - 1) * 6];
        self.triangles_index = 0;

        let mut vertex_index = 0;
        let mut y = 0;
        while y < self.map_width {
            let mut x = 0;
            while x < self.map_width {
                let height = self.height_map.data[y][x];

                self.vertices[vertex_index] = [x as f32, height as f32, y as f32];
                self.uvs[vertex_index] = [
                    x as f32 / self.map_width as f32,
                    y as f32 / self.map_width as f32,
                ];

                if x < self.map_width - 1 && y < self.map_width - 1 {
                    let top_left = vertex_index;
                    let top_right = vertex_index + 1;
                    let bottom_left = vertex_index + self.vertices_per_line;
                    let bottom_right = vertex_index + self.vertices_per_line + 1;
                    self.add_triangle(bottom_right, top_left, bottom_left);
                    self.add_triangle(top_left, bottom_right, top_right);
                }

                vertex_index += 1;
                x += self.simplification_increment;
            }
            y += self.simplification_increment;
        }
        self.calculate_normals();
    }

    fn add_triangle(&mut self, a: usize, b: usize, c: usize) {
        self.triangles[self.triangles_index as usize] = a as u32;
        self.triangles[(self.triangles_index + 1) as usize] = b as u32;
        self.triangles[(self.triangles_index + 2) as usize] = c as u32;
        self.triangles_index += 3;
    }

    pub fn graphics_mesh(&mut self) -> Mesh {
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

    pub fn collider_shape(&self) -> ColliderShape {
        let iter = self.vertices.iter().map(|&[_, y, _]| 0.0);
        let heights = DMatrix::from_iterator(self.map_width, self.map_width, iter);
        let scale = Vector3::new(self.map_width as f32, 1.0, self.map_width as f32);

        SharedShape::heightfield(heights, scale)
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
