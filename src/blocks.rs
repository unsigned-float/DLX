//! Block generation.

use std::collections::HashSet;
use crate::Node;

/// Blocks in 2D.
#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Block2D {
    pub w: usize,
    pub h: usize,
    pub data: Vec<Vec<bool>>,
}
impl Block2D {
    /// Create a block from a string. Empty squares are '.', filled are anything else.
    fn from_string(s: &str) -> Block2D {
        let mut data: Vec<Vec<bool>> = Vec::new();

        for line in s.trim().lines() {
            let mut row: Vec<bool> = Vec::new();
            for ch in line.trim().chars() {
                row.push(ch != '.');
            }
            data.push(row);
        }

        Block2D {
            w: data[0].len(),
            h: data.len(),
            data,
        }
    }

    /// Get the flipped form of a block.
    fn flip(&mut self) {
        self.data = self.data.iter().rev().cloned().collect();
    }

    /// Get the rotated form of a block.
    fn rotate(&mut self) {
        self.data = (0..self.w).map(|y|
            (0..self.h).rev().map(|x| self.data[x][y]).collect()
        ).collect();
        std::mem::swap(&mut self.w, &mut self.h);
    }

    /// Get all the unique transformations of a block within a grid.
    fn get_transformations(&mut self) -> Vec<Block2D> {
        let mut hs: HashSet<Block2D> = HashSet::new();
        for _ in 0..2 {
            for _ in 0..4 {
                hs.insert(self.clone());
                self.rotate();
            }
            self.flip();
        }
        hs.into_iter().collect()
    }
}

/// A container for blocks, bounded with a width and height.
pub struct Game2D {
    pub w: usize,
    pub h: usize,
    pub blocks: Vec<Block2D>,
}
impl Game2D {
    /// Create a game from a width, height, and vector of strings.
    pub fn from_strings(w: usize, h: usize, s: Vec<&str>) -> Game2D {
        let mut blocks: Vec<Block2D> = Vec::new();
        for block in s {
            blocks.push(Block2D::from_string(block));
        }

        Game2D { w, h, blocks }
    }

    /// Create a matrix from the blocks in the game to use within DLX and create the structure.
    pub fn get_matrix(&mut self) -> Vec<Vec<bool>> {
        let mut variations = Vec::new();

        for block in self.blocks.iter_mut() {
            for transformation in block.get_transformations() {
                for shift_y in 0..=(self.h - transformation.h) {
                    for shift_x in 0..=(self.w - transformation.w) {
                        let mut entry = Vec::new();
                        for py in 0..self.h {
                            for px in 0..self.w {
                                entry.push(
                                    shift_x <= px &&
                                    px < (shift_x + transformation.w) &&
                                    shift_y <= py &&
                                    py < (shift_y + transformation.h) &&
                                    transformation.data[py - shift_y][px - shift_x]
                                );
                            }
                        }
                        variations.push(entry);
                    }
                }
            }
        }

        Node::matrix_from_variations(&variations)
    }
}
