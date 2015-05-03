extern crate png;

use png::PixelsByColorType;
use std::iter::repeat;
use std::path;

pub struct Image {
    pub pixel_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

#[derive(Copy, Clone)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        self.red == other.red &&
        self.green == other.green &&
        self.blue == other.blue
    }
}

impl Image {
    pub fn load_png(path: &path::Path) -> Image {
        let image = match png::load_png(path) {
            Err(m) => panic!(m),
            Ok(image) => image,
        };
        println!("File dimensions: (width, height) = ({}, {}).", image.width, image.height);
        let pixel_data = match image.pixels {
            PixelsByColorType::RGBA8(pixels) => pixels,
            _ => panic!("Only handling RGBA8 input for now."),
        };
        Image {
            pixel_data: pixel_data,
            width: image.width,
            height: image.height,
        }
    }

    pub fn save_png(&self, path: &path::Path) {
        let mut img = png::Image {
            width: self.width,
            height: self.height,
            pixels: PixelsByColorType::RGBA8(self.pixel_data.clone()),
        };
        let res = png::store_png(&mut img, path);
        assert!(res.is_ok());
    }

    pub fn white(width: u32, height: u32) -> Image {
        Image {
            width: width,
            height: height,
            pixel_data: repeat(255u8).take((width * height * 4) as usize).collect(),
        }
    }

    pub fn color_at(&self, point: Point) -> Color {
        let pixel_offset = self.linear_index(point) * 4;
        Color{
            red: self.pixel_data[pixel_offset],
            green: self.pixel_data[pixel_offset + 1],
            blue: self.pixel_data[pixel_offset + 2],
        }
    }

    pub fn set_color_at(&mut self, point: Point, color: Color) {
        let pixel_offset = self.linear_index(point) * 4;
        self.pixel_data[pixel_offset] = color.red;
        self.pixel_data[pixel_offset + 1] = color.green;
        self.pixel_data[pixel_offset + 2] = color.blue;
    }

    pub fn linear_index(&self, point: Point) -> usize {
        point.y as usize * self.width as usize + point.x as usize
    }
}