extern crate collections;
extern crate png;

use collections::Deque;
use collections::RingBuf;

use png::load_png;
use png::RGBA8;

struct Point {
    x: int,
    y: int,
}

struct Cell {
    color: Color
}

struct Color {
    red: u8,
    green: u8,
    blue: u8,
}

impl PartialEq for Color {
    fn eq(&self, other: &Color) -> bool {
        self.red == other.red &&
        self.green == other.green &&
        self.blue == other.blue
    }
}

fn main() {
    // Load example PNG image.
    let file = "examples/cartesian_grid.png";
    println!("Loading '{}'.", file);
    let image = match load_png(&Path::new(file)) {
        Err(m) => fail!(m),
        Ok(image) => image,
    };
    println!("File dimensions: (width, height) = ({}, {}).", image.width, image.height);
    let pixel_data = match image.pixels {
        RGBA8(pixels) => pixels,
        _ => fail!("Only handling RGBA8 input for now."),
    };

    // Explore image breadth-first to break it
    // into cells of the same color.
    println!("Finding cells in image...");
    let pixels = (image.width * image.height) as uint;
    let mut cell_map: Vec<Option<uint>> = Vec::from_elem(pixels, None);
    let mut cells: Vec<Cell> = Vec::with_capacity(100);
    let mut point_queue: RingBuf<Point> = RingBuf::with_capacity(pixels);
    // Create per-cell scratch space here so we don't need to allocate
    // again for every cell we visit.
    let mut cell_point_queue: RingBuf<Point> = RingBuf::with_capacity(pixels);
    point_queue.push(Point{ x: 0, y: 0 });
    while !point_queue.is_empty() {
        let point = match point_queue.pop_front() {
            None => fail!(),
            Some(p) => p,
        };
        let image_width = image.width as int;
        // It might have already been consumed by another cell.
        if cell_map[index(point, image_width)] == None {
            cells.push(Cell{ color: color_of_point(point, image_width, &pixel_data) });
            let cell_index = cells.len() - 1;
            *cell_map.get_mut(index(point, image_width)) = Some(cell_index);
            let cell = cells.get_mut(cell_index);

            // Need to explore a single cell exhaustively before moving on;
            // otherwise we might interpret a strangely shaped cell as two
            // different cells and then have to join them up somehow later.
            flood_cell(
                &mut cell_map,
                &mut point_queue,
                &mut cell_point_queue,
                point,
                cell,
                cell_index,
                image_width,
                image.height as int,
                &pixel_data
            );
        }
    }
    println!("Found {} cells.", cells.len());
}

fn flood_cell(
    cell_map: &mut Vec<Option<uint>>,
    point_queue: &mut RingBuf<Point>,
    cell_point_queue: &mut RingBuf<Point>,
    starting_point: Point,
    cell: &mut Cell,
    cell_index: uint,
    image_width: int,
    image_height: int,
    pixel_data: &Vec<u8>
) {
    cell_point_queue.clear();
    cell_point_queue.push(starting_point);
    while !cell_point_queue.is_empty() {
        let point = match cell_point_queue.pop_front() {
            None => fail!(),
            Some(p) => p,
        };
        let neighbors = point_neighbors(point);
        for neighbor in neighbors.iter() {
            // Ignore if outside the image bounds.
            // TODO: support wrapping, and otherwise mark
            // this as the edge of the cell.
            if neighbor.x < 0 { continue; }
            if neighbor.x >= image_width { continue; }
            if neighbor.y < 0 { continue; }
            if neighbor.y >= image_height { continue; }

            let neighbor_cell = cell_map.get_mut(index(*neighbor, image_width));
            match *neighbor_cell {
                None => {
                    if color_of_point(*neighbor, image_width, pixel_data) == cell.color {
                        // Same color as this cell; add it to the cell and queue it
                        // up as a starting point for further explanation.
                        *neighbor_cell = Some(cell_index);
                        cell_point_queue.push(*neighbor);
                    } else {
                        // Doesn't belong to this cell; queue it up to maybe
                        // be the start of another cell.
                        point_queue.push(*neighbor);

                        // TODO: mark this as the edge of the current cell.
                    }
                },
                Some(int) => {
                    // TODO: mark this as the edge of the current cell.
                }
            }
        }
    }
}

fn color_of_point(point: Point, image_width: int, pixel_data: &Vec<u8>) -> Color {
    let pixel_offset = index(point, image_width) * 4;
    Color{
        red: pixel_data[pixel_offset],
        green: pixel_data[pixel_offset + 1],
        blue: pixel_data[pixel_offset + 2],
    }
}

fn index(point: Point, image_width: int) -> uint {
    (point.y * image_width + point.x) as uint
}

fn point_neighbors(point: Point) -> [Point, ..8] {
    [
        Point{ x: point.x - 1, y: point.y - 1 },
        Point{ x: point.x,     y: point.y - 1 },
        Point{ x: point.x + 1, y: point.y - 1 },
        Point{ x: point.x - 1, y: point.y     },
        Point{ x: point.x + 1, y: point.y     },
        Point{ x: point.x - 1, y: point.y + 1 },
        Point{ x: point.x,     y: point.y + 1 },
        Point{ x: point.x + 1, y: point.y + 1 },
    ]
}