use std::iter::repeat;
use std::collections::vec_deque::VecDeque;

use rand::{thread_rng, Rng};

use image::{Image, Color, Point};
use world::{World, Cell};

pub struct WorldBuilder {
    image: Image, // Source image.
    // Store separate boundaries image to blit over the top of the cell fills.
    cell_boundaries: Image,
    cell_map: Vec<Option<usize>>,
    // TODO: reinstate separate `visited` map, too, for faster checking?
    cells: Vec<Cell>,
    point_queue: VecDeque<Point>,
    // Per-cell scratch space so we don't need to allocate again for every cell we visit.
    cell_point_queue: VecDeque<Point>,
    wrap: bool,
    smin: u32,
    smax: u32,
    rmin: u32,
    rmax: u32,
    proportional: bool,
}

impl WorldBuilder {
    pub fn new(
        image: Image,
        wrap: bool,
        smin: u32,
        smax: u32,
        rmin: u32,
        rmax: u32,
        proportional: bool,
    ) -> WorldBuilder {
        let pixels = (image.width * image.height) as usize;
        let builder = WorldBuilder {
            cell_boundaries: Image::white(image.width, image.height),
            image: image,
            cell_map: repeat(None).take(pixels).collect(),
            cells: Vec::with_capacity(100),
            point_queue: VecDeque::with_capacity(pixels),
            cell_point_queue: VecDeque::with_capacity(pixels),
            wrap: wrap,
            smin: smin,
            smax: smax,
            rmin: rmin,
            rmax: rmax,
            proportional: proportional,
        };
        builder
    }

    pub fn build(mut self) -> World {
        // Explore image breadth-first to break it
        // into cells of the same color.
        println!("Finding cells in image...");

        self.point_queue.push_back(Point{ x: 0, y: 0 });
        while !self.point_queue.is_empty() {
            let point = match self.point_queue.pop_front() {
                None => panic!(),
                Some(p) => p,
            };
            // It might have already been consumed by another cell.
            if self.cell_map[self.image.linear_index(point)] == None {
                self.cells.push(Cell{
                    color: self.image.color_at(point),
                    neighbors: Vec::with_capacity(8),
                    pixels: Vec::with_capacity(100),
                });
                let cell_index = self.cells.len() - 1;
                self.cell_map[self.image.linear_index(point)] = Some(cell_index);

                // Need to explore a single cell exhaustively before moving on;
                // otherwise we might interpret a strangely shaped cell as two
                // different cells and then have to join them up somehow later.
                self.flood_cell(point, cell_index);
            }
        }
        println!("Found {} cells.", self.cells.len());

        // Randomise initial world state.
        let mut rng = thread_rng();
        let rng_iter = rng.gen_iter::<bool>();
        let world = World::new(
            rng_iter.take(self.cells.len()).collect(),
            repeat(false).take(self.cells.len()).collect(),
            Image::white(self.image.width, self.image.height),
            self.cell_boundaries,
            self.smin,
            self.smax,
            self.rmin,
            self.rmax,
            self.proportional,
            self.cells,
        );
        world
    }

    fn flood_cell(&mut self, starting_point: Point, cell_index: usize) {
        let cell_color = (*self.cells)[cell_index].color;
        self.cell_point_queue.clear();
        self.cell_point_queue.push_back(starting_point);
        while !self.cell_point_queue.is_empty() {
            let point = match self.cell_point_queue.pop_front() {
                None => panic!(),
                Some(p) => p,
            };
            self.cells[cell_index].pixels.push(point);
            let neighbors = point.neighbors();
            for neighbor in neighbors.iter() {
                let mut neighbor = *neighbor;

                // Wrap coordinates if requested.
                // Note that the result of % depends on the sign of the divisor,
                // so transpose everything north to avoid negative numbers entirely.
                if self.wrap {
                    neighbor = Point{
                        x: (neighbor.x + self.image.width as i32) % self.image.width as i32,
                        y: (neighbor.y + self.image.height as i32) % self.image.height as i32,
                    };
                }

                // Ignore if outside the image bounds.
                let oob = !self.wrap && (
                    neighbor.x < 0 ||
                    neighbor.x >= self.image.width as i32 ||
                    neighbor.y < 0 ||
                    neighbor.y >= self.image.height as i32
                );
                if oob {
                    // Mark the current pixel (not the neighbor) as the edge of a cell.
                    self.mark_cell_border(point);
                    continue;
                }

                let neighbor_cell = self.cell_map[self.image.linear_index(neighbor)];
                match neighbor_cell {
                    None => {
                        if self.image.color_at(neighbor) == cell_color {
                            // Same color as this cell; add it to the cell and queue it
                            // up as a starting point for further exploration.
                            let neighbor_cell = &mut self.cell_map[self.image.linear_index(neighbor)];
                            *neighbor_cell = Some(cell_index);
                            self.cell_point_queue.push_back(neighbor);
                        } else {
                            // Doesn't belong to this cell; queue it up to maybe
                            // be the start of another cell.
                            self.point_queue.push_back(neighbor);

                            // Neighbor is another color, so will eventually be part of another cell.
                            // Mark the current pixel (not the neighbor) as the edge of a cell.
                            self.mark_cell_border(point);
                        }
                    },
                    Some(neighbor_cell) => {
                        if self.image.color_at(neighbor) != cell_color {
                            // Neighbor is already part of another cell.
                            // Mark the current pixel (not the neighbor) as the edge of a cell.
                            self.mark_cell_border(point);

                            // Mark the cells as neighbors if they're not already.
                            if !(*self.cells)[cell_index].neighbors.contains(&neighbor_cell) {
                                self.cells[cell_index].neighbors.push(neighbor_cell);
                                self.cells[neighbor_cell].neighbors.push(cell_index);
                            }
                        }
                    }
                }
            }
        }
    }

    fn mark_cell_border(&mut self, point: Point) {
        self.cell_boundaries.set_color_at(point, Color{red: 127, green: 127, blue: 127});
    }
}
