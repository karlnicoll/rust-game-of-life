// MIT License
//
// Copyright (c) 2022 Karl Nicoll
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use rand;
use std::collections::HashSet;
use xy_utils::Point;

/// Cell Enumeration
///
/// Cells are the smallest atom of game state. The "game board" is made up of
/// a matrix of alive and dead cells. When the game rules are applied to the
/// cells on the board, each invividual cell may change to a new state.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Cell {
    Alive,
    Dead,
}

/// Trait used to define objects that can create new cells on a game board.
///
/// Objects implementing this trait can be passed to the game board to generate
/// cell patterns.
pub trait CellGenerator {
    /// Generate a new cell.
    ///
    /// ## Arguments
    ///
    /// * `address` The coordinates of the cell.
    fn generate(&mut self, address: Point) -> Cell;
}

/// GellGenerator trait implementation that generates a random cell state.
pub struct RandomCellGenerator<RandomT: rand::RngCore> {
    pub rng: RandomT,
}

impl<RandomT: rand::RngCore> CellGenerator for RandomCellGenerator<RandomT> {
    fn generate(&mut self, _: Point) -> Cell {
        if (self.rng.next_u64() % 2) == 0 {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }
}

/// Cell generator that uses a pre-defined pattern to generate the cells.
pub struct UserCellGenerator {
    alive_cells_list: HashSet<Point>,
}

impl CellGenerator for UserCellGenerator {
    fn generate(&mut self, address: Point) -> Cell {
        if self.alive_cells_list.contains(&address) {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }
}

impl UserCellGenerator {
    /// Create the starting layout from a string.
    ///
    /// ## Example
    ///
    /// ```
    /// let input =
    ///     "*****   *****    *****\n\
    ///      *      *     *  *     *\n\
    ///      ***    *     *  *     *\n\
    ///      *      *     *  *     *\n\
    ///      *       *****    *****"
    ///
    /// let gen = UserCellGenerator::from_str(input);
    /// ```
    pub fn from_str(s: &str) -> Result<UserCellGenerator, String> {
        let mut x = 0;
        let mut y = 0;
        let mut cell_set = HashSet::new();
        for c in s.chars() {
            match c {
                '*' => {
                    cell_set.insert(Point { x, y });
                    x += 1;
                }
                ' ' => {
                    x += 1;
                }
                '\n' => {
                    x = 0;
                    y += 1;
                }
                _ => {
                    return Err(format!(
                        "Invalid character '{}' specified in UserCellGenerator::from_str()",
                        c
                    ));
                }
            }
        }

        Ok(UserCellGenerator { alive_cells_list: cell_set })
    }
}

// =============================================================================

#[cfg(test)]
mod cell_tests {
    #[test]
    fn cells_are_copyable() {
        let cell = super::Cell::Alive;
        let copied_cell = cell;
        assert_eq!(cell, copied_cell);
    }
}

#[cfg(test)]
mod random_cell_generator_tests {
    use super::*;

    #[test]
    fn generates_random_cell_states() {
        use rand::rngs::mock::StepRng;
        let mut gen = RandomCellGenerator { rng: StepRng::new(0, 1) };

        // Using StepRng should produce a consistent true/false pattern. Cell
        // address doesn't matter for this generator.
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Dead);
    }
}

#[cfg(test)]
mod user_cell_generator_tests {
    use super::*;

    #[test]
    fn generates_cell_distribution_from_user_input() {
        // Generates a pattern like this:
        //
        // ** *
        //  **** *
        //  ****   **
        let mut gen = UserCellGenerator::from_str(
            "** * \n\
             **** *\n\
             ****   **",
        )
        .unwrap();

        // Perfectly alternates between dead and alive. Cell address DOES
        // matter for this generator. Results are row major ordered for easy
        // grokking.
        assert_eq!(gen.generate(Point { x: 0, y: 0 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 1, y: 0 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 2, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 3, y: 0 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 4, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 5, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 6, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 7, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 8, y: 0 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 0, y: 1 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 1, y: 1 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 2, y: 1 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 3, y: 1 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 4, y: 1 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 5, y: 1 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 6, y: 1 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 7, y: 1 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 8, y: 1 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 0, y: 2 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 1, y: 2 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 2, y: 2 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 3, y: 2 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 4, y: 2 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 5, y: 2 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 6, y: 2 }), Cell::Dead);
        assert_eq!(gen.generate(Point { x: 7, y: 2 }), Cell::Alive);
        assert_eq!(gen.generate(Point { x: 8, y: 2 }), Cell::Alive);
    }

    #[test]
    #[should_panic]
    fn invalid_characters_produce_an_error() {
        // This call to UserCellGenerator::from_str should report an error.
        UserCellGenerator::from_str("*** *** This_string_is_full_of_invalid_characters *** ***")
            .unwrap();
    }
}
