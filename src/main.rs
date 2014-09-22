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
    let pixels = (image.width * image.height) as uint;
    let mut visit_map: Vec<bool> = Vec::from_elem(pixels, false);
    let mut queue: RingBuf<Point> = RingBuf::with_capacity(pixels);
    enqueue_if_new(&mut visit_map, &mut queue, Point{ x: 0, y: 0 }, image.width as int);
    while !queue.is_empty() {
        let point = match queue.pop_front() {
            None => fail!(),
            Some(p) => p,
        };
        enqueue_new_neighbors(&mut visit_map, &mut queue, point, image.width as int, image.height as int)
    }
}

fn enqueue_new_neighbors(visit_map: &mut Vec<bool>, queue: &mut RingBuf<Point>, point: Point, image_width: int, image_height: int) {
    let neighbors = [
        Point{ x: point.x - 1, y: point.y - 1 },
        Point{ x: point.x,     y: point.y - 1 },
        Point{ x: point.x + 1, y: point.y - 1 },
        Point{ x: point.x - 1, y: point.y     },
        Point{ x: point.x + 1, y: point.y     },
        Point{ x: point.x - 1, y: point.y + 1 },
        Point{ x: point.x,     y: point.y + 1 },
        Point{ x: point.x + 1, y: point.y + 1 },
    ];
    for neighbor in neighbors.iter() {
        // Ignore if outside the image bounds.
        // TODO: support wrapping.
        if neighbor.x < 0 { continue; }
        if neighbor.x >= image_width { continue; }
        if neighbor.y < 0 { continue; }
        if neighbor.y >= image_height { continue; }
        enqueue_if_new(visit_map, queue, *neighbor, image_width);
    }
}

fn enqueue_if_new(visit_map: &mut Vec<bool>, queue: &mut RingBuf<Point>, point: Point, image_width: int) {
    let visited = visit_map.get_mut(index(point, image_width));
    if !*visited {
        *visited = true;
        queue.push(point);
    }
}

fn index(point: Point, image_width: int) -> uint {
    (point.y * image_width + point.x) as uint
}