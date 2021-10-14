use bevy::{
    prelude::*,
    render::texture::{Extent3d, TextureDimension, TextureFormat},
};

use super::{height_map::HeightMap, Config};

pub fn generate(height_map: &HeightMap, config: &Config) -> Texture {
    let color_map = generate_color_map(height_map, config);
    return generate_texture(&color_map);
}

fn generate_color_map(height_map: &HeightMap, config: &Config) -> ColorMap {
    let mut color_map = ColorMap::new((height_map.size, height_map.size));
    for y in 0..height_map.size {
        for x in 0..height_map.size {
            let height = height_map.data[y][x];

            for terrain in config.terrain_thresholds.iter() {
                if height < terrain.max_height {
                    color_map.colors.push(terrain.color);
                    break;
                }
            }
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
