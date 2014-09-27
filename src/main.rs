extern crate collections;
extern crate png;
extern crate getopts;

use std::os;
use getopts::{optopt, optflag, getopts, OptGroup};

use collections::Deque;
use collections::RingBuf;

use image::{Image, Color, Point};

mod image;

struct Cell {
    color: Color
}

fn main() {
    // Parse program arguments.
    let args: Vec<String> = os::args();
    let opts = [
        optflag("w", "wrap", "treat image space as toroidal")
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { fail!(f.to_string()) }
    };
    let wrap = matches.opt_present("w");

    // Load example PNG image.
    let file = "examples/hex_square_tri.png";
    println!("Loading '{}'.", file);
    let image = Image::load_png(&Path::new(file));

    // Draw discovered cell boundaries into a new image.
    let mut cell_boundaries = Image::white(image.width, image.height);

    // Explore image breadth-first to break it
    // into cells of the same color.
    println!("Finding cells in image...");
    let pixels = (image.width * image.height) as uint;
    // TODO: reinstate separate `visited` map, too, for faster checking?
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
        // It might have already been consumed by another cell.
        if cell_map[image.linear_index(point)] == None {
            cells.push(Cell{ color: image.color_at(point) });
            let cell_index = cells.len() - 1;
            *cell_map.get_mut(image.linear_index(point)) = Some(cell_index);
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
                &image,
                &mut cell_boundaries,
                wrap,
            );
        }
    }
    println!("Found {} cells.", cells.len());

    // Write out discovered cell boundaries.
    cell_boundaries.save_png(&Path::new("image_out/cell_boundaries.png"));
}

fn flood_cell(
    cell_map: &mut Vec<Option<uint>>,
    point_queue: &mut RingBuf<Point>,
    cell_point_queue: &mut RingBuf<Point>,
    starting_point: Point,
    cell: &mut Cell,
    cell_index: uint,
    image: &Image,
    cell_boundaries: &mut Image,
    wrap: bool,
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
            let mut neighbor = *neighbor;

            // Wrap coordinates if requested.
            // Note that the result of % depends on the sign of the divisor,
            // so transpose everything north to avoid negative numbers entirely.
            if wrap {
                neighbor = Point{
                    x: (neighbor.x + image.width as i32) % image.width as i32,
                    y: (neighbor.y + image.height as i32) % image.height as i32,
                };
            }

            // Ignore if outside the image bounds.
            let oob = !wrap && (
                neighbor.x < 0 ||
                neighbor.x >= image.width as i32 ||
                neighbor.y < 0 ||
                neighbor.y >= image.height as i32
            );
            if oob {
                // Mark the current pixel (not the neighbor) as the edge of a cell.
                mark_cell_border(point, cell_boundaries);
                continue;
            }

            let neighbor_cell = cell_map.get_mut(image.linear_index(neighbor));
            match *neighbor_cell {
                None => {
                    if image.color_at(neighbor) == cell.color {
                        // Same color as this cell; add it to the cell and queue it
                        // up as a starting point for further explanation.
                        *neighbor_cell = Some(cell_index);
                        cell_point_queue.push(neighbor);
                    } else {
                        // Doesn't belong to this cell; queue it up to maybe
                        // be the start of another cell.
                        point_queue.push(neighbor);

                        // Neighbor is another color, so will eventually be part of another cell.
                        // Mark the current pixel (not the neighbor) as the edge of a cell.
                        mark_cell_border(point, cell_boundaries);
                    }
                },
                Some(int) => {
                    if image.color_at(neighbor) != cell.color {
                        // Neighbor is already part of another cell.
                        // Mark the current pixel (not the neighbor) as the edge of a cell.
                        mark_cell_border(point, cell_boundaries);
                    }
                }
            }
        }
    }
}

fn mark_cell_border(point: Point, cell_boundaries: &mut Image) {
    cell_boundaries.set_color_at(point, Color{red: 127, green: 127, blue: 127});
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