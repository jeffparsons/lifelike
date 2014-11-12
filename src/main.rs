extern crate core;
extern crate collections;
extern crate png;
extern crate getopts;

use core::mem;

use std::os;
use std::rand::{task_rng, Rng};

use collections::RingBuf;
use collections::String;

use getopts::{optopt, optflag, getopts, OptGroup, Matches, usage};

use image::{Image, Color, Point};
mod image;

struct Cell {
    color: Color,
    neighbors: Vec<uint>,
    pixels: Vec<Point>,
}

fn print_usage(program: &str, opts: &[OptGroup]) {
    let short_message = format!("Usage: {} [options] <input_file>", program);
    println!("{}", usage(short_message.as_slice(), opts));
}

fn get_uint_opt(matches: &Matches, opt_name: &str) -> Option<uint> {
    match matches.opt_str(opt_name) {
        Some(string) => match from_str(string.as_slice().trim()) {
            Some(value) => Some(value),
            None => panic!("Bad uint arg"),
        },
        None => None,
    }
}

fn main() {
    // Parse program arguments.
    let args: Vec<String> = os::args();
    let program = args[0].clone();
    let opts = [
        optopt("", "smin", "minimum neighbors for existing cell to survive", "UINT"),
        optopt("", "smax", "maximum neighbors for existing cell to survive", "UINT"),
        optopt("", "rmin", "minimum neighbors for new cell to be born", "UINT"),
        optopt("", "rmax", "maximum neighbors for new cell to be born", "UINT"),
        optopt("f", "frames", "number of frames to render", "UINT"),
        optflag("w", "wrap", "treat image space as toroidal"),
        optflag("p", "proportional", "weight neighbors by how many neighbors they have"),
        optopt("o", "output-prefix", "prefix for output frame files", "STRING"),
        optflag("h", "help", "print usage information"),
    ];
    let matches = match getopts(args.tail(), opts) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(program.as_slice(), opts);
        return;
    }
    let input = if matches.free.len() == 1 {
        matches.free[0].clone()
    } else {
        print_usage(program.as_slice(), opts);
        return;
    };
    let output_prefix = matches.opt_str("output-prefix").unwrap_or(String::from_str("frame_"));

    let frames = get_uint_opt(&matches, "frames").unwrap_or(100);
    let wrap = matches.opt_present("w");
    let proportional = matches.opt_present("p");

    let smin = get_uint_opt(&matches, "smin").unwrap_or(2);
    let smax = get_uint_opt(&matches, "smax").unwrap_or(3);
    let rmin = get_uint_opt(&matches, "rmin").unwrap_or(3);
    let rmax = get_uint_opt(&matches, "rmax").unwrap_or(3);

    // Load example PNG image.
    // let file = "examples/hex_square_tri_large.png";
    // let file = "examples/cartesian_grid.png";
    // let file = "examples/hex_grid.png";
    println!("Loading '{}'.", input);
    let image = Image::load_png(&Path::new(input));

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
    point_queue.push_back(Point{ x: 0, y: 0 });
    while !point_queue.is_empty() {
        let point = match point_queue.pop_front() {
            None => panic!(),
            Some(p) => p,
        };
        // It might have already been consumed by another cell.
        if cell_map[image.linear_index(point)] == None {
            cells.push(Cell{
                color: image.color_at(point),
                neighbors: Vec::with_capacity(8),
                pixels: Vec::with_capacity(100),
            });
            let cell_index = cells.len() - 1;
            cell_map[image.linear_index(point)] = Some(cell_index);

            // Need to explore a single cell exhaustively before moving on;
            // otherwise we might interpret a strangely shaped cell as two
            // different cells and then have to join them up somehow later.
            flood_cell(
                &mut cells,
                &mut cell_map,
                &mut point_queue,
                &mut cell_point_queue,
                point,
                cell_index,
                &image,
                &mut cell_boundaries,
                wrap,
            );
        }
    }
    println!("Found {} cells.", cells.len());

    // Write out discovered cell boundaries.
    // cell_boundaries.save_png(&Path::new("image_out/cell_boundaries.png"));

    // Randomise initial world state.
    let mut rng = task_rng();
    let rng_iter = rng.gen_iter::<bool>();
    let mut front: &mut Vec<bool> = &mut rng_iter.take(cells.len()).collect::<Vec<bool>>();

    // Make back buffer for world.
    let mut back: &mut Vec<bool> = &mut Vec::from_elem(cells.len(), false);

    // Step world, writing out an image of the current state
    // at the start of each frame.
    let mut world_image = Image::white(image.width, image.height);
    for frame in range(0, frames) {
        // Write out current state.
        for (i, cell) in cells.iter().enumerate() {
            for p in cell.pixels.iter() {
                let color = if (*front)[i] {
                    Color{ red: 63, green: 63, blue: 63 }
                } else {
                    Color{ red: 255, green: 255, blue: 255 }
                };
                world_image.set_color_at(*p, color);
            }
        }
        // Overlay cell boundaries.
        for y in range(0, image.height as i32) {
            for x in range(0, image.width as i32) {
                let p = Point{ x: x, y: y };
                let color_in_boundary_image = cell_boundaries.color_at(p);
                let white = Color{ red: 255, green: 255, blue: 255 };
                if color_in_boundary_image != white {
                    world_image.set_color_at(p, color_in_boundary_image);
                }
            }
        }

        let frame_file = format!("image_out/{}{:0>8}.png", output_prefix, frame);
        println!("Writing frame to '{}'.", frame_file);
        world_image.save_png(&Path::new(frame_file));

        // Calculate next frame.
        for i in range(0, cells.len()) {
            let alive = (*front)[i];
            let mut living_neighbors = 0u;
            for neighbor in cells[i].neighbors.iter() {
                if (*front)[*neighbor] {
                    if proportional {
                        living_neighbors += cells[*neighbor].neighbors.len();
                    } else {
                        living_neighbors += 1;
                    }
                }
            }
            if proportional {
                living_neighbors = living_neighbors * 4 / cells[i].neighbors.len();
            }

            // Apply life rules.
            back[i] = if alive {
                living_neighbors >= smin && // Sufficient neighbors to sustain.
                living_neighbors <= smax // Not so many we're overcrowded.
            } else {
                living_neighbors >= rmin && // Sufficient neighbors to reproduce.
                living_neighbors <= rmax // Not so many we're overcrowded;
                                         // possible to make higher than smax to
                                         // give unfair advantage to newborns.
            }
        }

        mem::swap(&mut front, &mut back);
    }
}

fn flood_cell(
    cells: &mut Vec<Cell>,
    cell_map: &mut Vec<Option<uint>>,
    point_queue: &mut RingBuf<Point>,
    cell_point_queue: &mut RingBuf<Point>,
    starting_point: Point,
    cell_index: uint,
    image: &Image,
    cell_boundaries: &mut Image,
    wrap: bool,
) {
    let cell_color = (*cells)[cell_index].color;
    cell_point_queue.clear();
    cell_point_queue.push_back(starting_point);
    while !cell_point_queue.is_empty() {
        let point = match cell_point_queue.pop_front() {
            None => panic!(),
            Some(p) => p,
        };
        cells[cell_index].pixels.push(point);
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

            let neighbor_cell = &mut cell_map[image.linear_index(neighbor)];
            match *neighbor_cell {
                None => {
                    if image.color_at(neighbor) == cell_color {
                        // Same color as this cell; add it to the cell and queue it
                        // up as a starting point for further explanation.
                        *neighbor_cell = Some(cell_index);
                        cell_point_queue.push_back(neighbor);
                    } else {
                        // Doesn't belong to this cell; queue it up to maybe
                        // be the start of another cell.
                        point_queue.push_back(neighbor);

                        // Neighbor is another color, so will eventually be part of another cell.
                        // Mark the current pixel (not the neighbor) as the edge of a cell.
                        mark_cell_border(point, cell_boundaries);
                    }
                },
                Some(neighbor_cell) => {
                    if image.color_at(neighbor) != cell_color {
                        // Neighbor is already part of another cell.
                        // Mark the current pixel (not the neighbor) as the edge of a cell.
                        mark_cell_border(point, cell_boundaries);

                        // Mark the cells as neighbors if they're not already.
                        if !(*cells)[cell_index].neighbors.contains(&neighbor_cell) {
                            cells[cell_index].neighbors.push(neighbor_cell);
                            cells[neighbor_cell].neighbors.push(cell_index);
                        }
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