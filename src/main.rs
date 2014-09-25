extern crate collections;
extern crate png;

use collections::Deque;
use collections::RingBuf;

use image::{Image, Color, Point};

mod image;

struct Cell {
    color: Color
}

fn main() {
    // Load example PNG image.
    let file = "examples/cartesian_grid.png";
    println!("Loading '{}'.", file);
    let image = Image::load_png(&Path::new(file));

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
    image: &Image
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
            if neighbor.x >= image.width as i32 { continue; }
            if neighbor.y < 0 { continue; }
            if neighbor.y >= image.height as i32 { continue; }

            let neighbor_cell = cell_map.get_mut(image.linear_index(*neighbor));
            match *neighbor_cell {
                None => {
                    if image.color_at(*neighbor) == cell.color {
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