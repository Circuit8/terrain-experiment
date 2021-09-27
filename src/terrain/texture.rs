use bevy::{
    prelude::*,
    render::texture::{Extent3d, TextureDimension, TextureFormat},
};

use super::height_map::HeightMap;

pub fn generate(height_map: &HeightMap) -> Texture {
    let color_map = generate_color_map(height_map);
    return generate_texture(&color_map);
}

fn generate_color_map(height_map: &HeightMap) -> ColorMap {
    let mut color_map = ColorMap::new((height_map.len(), height_map.len()));
    for y in 0..height_map.len() {
        for x in 0..height_map.len() {
            let height = height_map[y][x];

            let color = if height < 0.35 {
                Color::rgb(0.0, 0.1, 0.8)
            } else if height < 0.43 {
                Color::rgb(0.9, 0.78, 0.01)
            } else if height < 0.85 {
                Color::rgb(0.01, 0.9, 0.05)
            } else {
                Color::rgb(0.65, 0.65, 0.65)
            };
            color_map.colors.push(color);
        }
    }
    return color_map;
}

fn generate_texture(color_map: &ColorMap) -> Texture {
    let mut image_buffer: Vec<u8> = vec![];

    for color in color_map.colors.iter() {
        image_buffer.push((color.r() * 255.) as u8);
        image_buffer.push((color.g() * 255.) as u8);
        image_buffer.push((color.b() * 255.) as u8);
        image_buffer.push(255);
    }

    Texture::new(
        Extent3d::new(color_map.size.0 as u32, color_map.size.1 as u32, 1),
        TextureDimension::D2,
        image_buffer,
        TextureFormat::Rgba8Unorm,
    )
}

#[derive(Default)]
struct ColorMap {
    pub colors: Vec<Color>,
    pub size: (usize, usize),
}

impl ColorMap {
    pub fn new(size: (usize, usize)) -> ColorMap {
        ColorMap {
            colors: vec![],
            size,
        }
    }
}
