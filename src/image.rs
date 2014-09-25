extern crate png;

use png::RGBA8;

pub struct Image {
    pub pixel_data: Vec<u8>,
    pub width: uint,
    pub height: uint,
}

pub struct Point {
    pub x: i32,
    pub y: i32,
}

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
    pub fn load_png(path: &Path) -> Image {
        let image = match png::load_png(path) {
            Err(m) => fail!(m),
            Ok(image) => image,
        };
        println!("File dimensions: (width, height) = ({}, {}).", image.width, image.height);
        let pixel_data = match image.pixels {
            RGBA8(pixels) => pixels,
            _ => fail!("Only handling RGBA8 input for now."),
        };
        Image {
            pixel_data: pixel_data,
            width: image.width as uint,
            height: image.height as uint,
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

    pub fn linear_index(&self, point: Point) -> uint {
        point.y as uint * self.width + point.x as uint
    }
}