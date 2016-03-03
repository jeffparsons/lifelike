extern crate png;
extern crate rand;

use std::mem;

use image::{Image, Color, Point};

pub struct Cell {
    pub color: Color,
    pub neighbors: Vec<usize>,
    pub pixels: Vec<Point>,
}

pub struct World {
    cells: Vec<Cell>,
    front: Vec<bool>,
    back: Vec<bool>,
    image: Image,
    cell_boundaries: Image,
    smin: u32,
    smax: u32,
    rmin: u32,
    rmax: u32,
    proportional: bool,
}

impl World {
    pub fn new(
        front: Vec<bool>,
        back: Vec<bool>,
        image: Image,
        cell_boundaries: Image,
        smin: u32,
        smax: u32,
        rmin: u32,
        rmax: u32,
        proportional: bool,
        cells: Vec<Cell>,
    ) -> World {
        World {
            cells: cells,
            front: front,
            back: back,
            image: image,
            cell_boundaries: cell_boundaries,
            smin: smin,
            smax: smax,
            rmin: rmin,
            rmax: rmax,
            proportional: proportional,
        }
    }

    pub fn update_world_image(&mut self) {
        // Write out current state.
        for (i, cell) in self.cells.iter().enumerate() {
            for p in cell.pixels.iter() {
                let color = if (*self.front)[i] {
                    Color{ red: 63, green: 63, blue: 63 }
                } else {
                    Color{ red: 255, green: 255, blue: 255 }
                };
                self.image.set_color_at(*p, color);
            }
        }
        // Overlay cell boundaries.
        for y in 0..self.image.height as i32 {
            for x in 0..self.image.width as i32 {
                let p = Point{ x: x, y: y };
                let color_in_boundary_image = self.cell_boundaries.color_at(p);
                let white = Color{ red: 255, green: 255, blue: 255 };
                if color_in_boundary_image != white {
                    self.image.set_color_at(p, color_in_boundary_image);
                }
            }
        }
    }

    pub fn step(&mut self) {
        // Calculate next frame.
        for i in 0..self.cells.len() {
            let alive = (*self.front)[i];
            let mut living_neighbors = 0u32;
            for neighbor in self.cells[i].neighbors.iter() {
                if (*self.front)[*neighbor] {
                    if self.proportional {
                        living_neighbors += self.cells[*neighbor].neighbors.len() as u32;
                    } else {
                        living_neighbors += 1;
                    }
                }
            }
            if self.proportional {
                living_neighbors = living_neighbors * 4 / self.cells[i].neighbors.len() as u32;
            }

            // Apply life rules.
            self.back[i] = if alive {
                living_neighbors >= self.smin && // Sufficient neighbors to sustain.
                living_neighbors <= self.smax // Not so many we're overcrowded.
            } else {
                living_neighbors >= self.rmin && // Sufficient neighbors to reproduce.
                living_neighbors <= self.rmax // Not so many we're overcrowded;
                                              // possible to make higher than smax to
                                              // give unfair advantage to newborns.
            }
        }

        mem::swap(&mut self.front, &mut self.back);
    }

    // We'll want to borrow this to write the state to disk.
    pub fn image(&self) -> &Image {
        &self.image
    }
}
