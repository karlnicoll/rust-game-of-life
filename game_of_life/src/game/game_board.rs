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

use crate::game::{Cell, CellGenerator, Renderer};
use xy_utils::{Dimensions, Point};

/// The Game Board.
///
/// The game board struct is used to hold the dimensions of the game, as well as
/// the individual cell states.
pub struct GameBoard<'a, RendererT: Renderer> {
    /// The individual cells on the game board, presented as a flat list.
    /// Ordering is column major, meaning that the list presents this matrix:
    ///
    /// 1 2 3
    /// 4 5 6
    /// 7 8 9
    ///
    /// As:
    ///
    /// 1 2 3 4 5 6 7 8 9-
    cells: Vec<Cell>,

    /// Width of the game board.
    dimensions: Dimensions,

    /// Renderer is used to print the game progress to a user interface.
    renderer: &'a mut RendererT,
}

impl<'a, RendererT: Renderer> GameBoard<'a, RendererT> {
    /// Create a new game board.
    ///
    /// ## Arguments
    ///
    /// * `dimensions`: Size of the game board.
    /// * `cell_generator`: Generator object that creates the initial cell
    ///   states.
    /// * `renderer`: Renderer that will output the state of the game board
    ///   after each iteration. Renderer should be initialized.
    pub fn new_from_seed<CellGeneratorT: CellGenerator>(
        dimensions: Dimensions,
        mut cell_generator: CellGeneratorT,
        renderer: &'a mut RendererT,
    ) -> GameBoard<'a, RendererT> {
        let mut cells = Vec::<Cell>::with_capacity(dimensions.total_area());
        for y in 0..dimensions.height {
            for x in 0..dimensions.width {
                cells.push(cell_generator.generate(Point { x, y }));
            }
        }

        // Apply initial set of alive cells directly to the renderer.
        let mut cells_to_render = Vec::<(Point, Cell)>::new();
        for (i, cell) in cells.iter().enumerate() {
            match cell {
                Cell::Alive => cells_to_render
                    .push((Self::get_cell_address_from_array_index(i, dimensions), *cell)),
                Cell::Dead => {}
            }
        }
        renderer.apply_changes(cells_to_render);

        GameBoard::<'a> { cells, dimensions, renderer }
    }

    pub fn calculate_iteration(&mut self) {
        let mut new_cells = Vec::<Cell>::with_capacity(self.cells.capacity());
        let mut cells_to_render = Vec::<(Point, Cell)>::new();
        for (i, cell) in self.cells.iter().enumerate() {
            let cell_address = Self::get_cell_address_from_array_index(i, self.dimensions);
            let new_cell_state = self.calculate_new_cell_state(cell_address, *cell);

            if new_cell_state != *cell {
                cells_to_render.push((cell_address, new_cell_state));
            }

            new_cells.push(new_cell_state);
        }

        self.cells = new_cells;
        self.renderer.apply_changes(cells_to_render);
    }

    /// For a given cell, calculate it's new state based on it's adjacent cells.
    fn calculate_new_cell_state(&self, cell_address: Point, cell: Cell) -> Cell {
        let alive_adjacents = self.count_alive_adjacent_cells(cell_address);

        // Rules of Conway's Game of Life:
        //
        // 1. A live cell with fewer than two live neighbours dies.
        // 2. A live cell with two or three live neighbours lives on.
        // 3. A live cell with more than three live neighbours dies.
        // 4. A dead cell with exactly three live neighbours becomes a live
        //    cell.

        match cell {
            Cell::Alive => {
                if (alive_adjacents == 2) || (alive_adjacents == 3) {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            }
            Cell::Dead => {
                if alive_adjacents == 3 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            }
        }
    }

    /// Cound the number of adjacent cells that are alive.
    fn count_alive_adjacent_cells(&self, cell_address: Point) -> usize {
        let mut count = 0;
        for adjacent_cell_address in self.calculate_adjacent_cell_addresses(cell_address) {
            let array_index =
                (adjacent_cell_address.y * self.dimensions.width) + (adjacent_cell_address.x);

            match self.cells[array_index] {
                Cell::Alive => count += 1,
                _ => {}
            }
        }

        count
    }

    /// Locate the adjacent cell addresses for a given cell address.
    ///
    /// Adjacent cells wrap around, so if the game board is 10x10, the adjacent
    /// cells for {0, 0} would be:
    ///
    /// * {x: 1, y: 0}
    /// * {x: 9, y: 0}
    /// * {x: 0, y: 1}
    /// * {x: 1, y: 1}
    /// * {x: 9, y: 1}
    /// * {x: 0, y: 9}
    /// * {x: 1, y: 9}
    /// * {x: 9, y: 9}
    fn calculate_adjacent_cell_addresses(&self, cell_address: Point) -> [Point; 8] {
        // Work out the adjacent cells. On edges (i.e. x or y is zero, or max
        // x or max y), the cell address will wrap around to the opposite edge
        // of the board.
        let row_above_y =
            if cell_address.y == 0 { self.dimensions.height - 1 } else { cell_address.y - 1 };
        let column_left_x =
            if cell_address.x == 0 { self.dimensions.width - 1 } else { cell_address.x - 1 };
        let column_right_x =
            if cell_address.x == (self.dimensions.width - 1) { 0 } else { cell_address.x + 1 };
        let row_below_y =
            if cell_address.y == (self.dimensions.height - 1) { 0 } else { cell_address.y + 1 };

        [
            // Top row
            Point { x: column_left_x, y: row_above_y },
            Point { x: cell_address.x, y: row_above_y },
            Point { x: column_right_x, y: row_above_y },
            // Middle row
            Point { x: column_left_x, y: cell_address.y },
            Point { x: column_right_x, y: cell_address.y },
            // Bottom row.
            Point { x: column_left_x, y: row_below_y },
            Point { x: cell_address.x, y: row_below_y },
            Point { x: column_right_x, y: row_below_y },
        ]
    }

    /// Convert a given index to a cell address.
    fn get_cell_address_from_array_index(i: usize, game_board_size: Dimensions) -> Point {
        Point { x: i % game_board_size.width, y: i / game_board_size.width }
    }
}

#[cfg(test)]
#[rustfmt::skip]  // Skipping rustfmt here because otherwise the cell layouts are unreadable.
mod game_board_tests {
    use super::*;
    use crate::game::{renderer::mock::MockRenderer, Renderer, UserCellGenerator};

    #[test]
    fn initializes() {
        let mut renderer = MockRenderer::new();

        // Game board should have provided the initial render state to the
        // renderer.
        let expected = concat!(
            "* * *\n",
            " * * \n",
            "* * *\n",
            " * * \n",
            "* * *"
        );

        {
            // Scope the game board so that the renderer borrow is returned for
            // introspection.
            GameBoard::new_from_seed(
                renderer.get_grid_size(), UserCellGenerator::from_str(expected).unwrap(), &mut renderer
            );
        }

        // Game board should have provided the initial render state to the
        // renderer.
        let expected = concat!(
            "* * *\n",
            " * * \n",
            "* * *\n",
            " * * \n",
            "* * *"
        );

        assert_eq!(expected, renderer.print_grid());
    }

    /// The following patterns do not change between game iterations.
    mod still_lifes {
        use crate::game::UserCellGenerator;

        use super::*;

        #[test]
        fn block() {
            let mut renderer = MockRenderer::new_with_size(Dimensions { width: 4, height: 4 });

            // Initial board state.
            let expected = concat!(
                "    \n",
                " ** \n",
                " ** \n",
                "    ",
            );

            {
                // Scope the game board so that the renderer borrow is returned for
                // introspection.
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(expected).unwrap(),
                    &mut renderer
                );

                // Run one iteration of the game. The output should not change.
                game_board.calculate_iteration();
            }

            assert_eq!(expected, renderer.print_grid());
        }

        #[test]
        fn beehive() {
            let mut renderer = MockRenderer::new_with_size(Dimensions { width: 6, height: 5 });

            let expected = concat!(
                "      \n",
                "  **  \n",
                " *  * \n",
                "  **  \n",
                "      "
            );

            {
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(expected).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(expected, renderer.print_grid());
        }

        #[test]
        fn loaf() {
            let mut renderer = MockRenderer::new_with_size(Dimensions { width: 6, height: 6 });

            let expected = concat!(
                "      \n",
                "  **  \n",
                " *  * \n",
                "  * * \n",
                "   *  \n",
                "      "
            );

            {
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(expected).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(expected, renderer.print_grid());
        }

        #[test]
        fn boat() {
            let mut renderer = MockRenderer::new_with_size(Dimensions { width: 5, height: 5 });

            let expected = concat!(
                "     \n",
                " **  \n",
                " * * \n",
                "  *  \n",
                "     "
            );

            {
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(expected).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(expected, renderer.print_grid());
        }

        #[test]
        fn tub() {
            let mut renderer = MockRenderer::new_with_size(Dimensions { width: 5, height: 5 });

            let expected = concat!(
                "     \n",
                "  *  \n",
                " * * \n",
                "  *  \n",
                "     "
            );

            {
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(expected).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(expected, renderer.print_grid());
        }
    }

    /// The following patterns change their pattern in-place, but return to
    /// their original form.
    mod oscillators {
        use crate::game::UserCellGenerator;
        use super::*;

        #[test]
        fn blinker() {
            let mut renderer = MockRenderer::new_with_size(Dimensions { width: 5, height: 5 });

            let initial = concat!(
                "     \n",
                "     \n",
                " *** \n",
                "     \n",
                "     "
            );

            let end = concat!(
                "     \n",
                "  *  \n",
                "  *  \n",
                "  *  \n",
                "     "
            );

            {
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(initial).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(end, renderer.print_grid());
        }

        #[test]
        fn toad() {
            let mut renderer = MockRenderer::new_with_size(Dimensions { width: 6, height: 6 });

            let initial = concat!(
                "      \n",
                "      \n",
                "  *** \n",
                " ***  \n",
                "      \n",
                "      "
            );

            let end = concat!(
                "      \n",
                "   *  \n",
                " *  * \n",
                " *  * \n",
                "  *   \n",
                "      "
            );

            {
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(initial).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(end, renderer.print_grid());
        }

        #[test]
        fn beacon() {
            let mut renderer = MockRenderer::new_with_size(Dimensions { width: 6, height: 6 });

            let initial = concat!(
                "      \n",
                " **   \n",
                " *    \n",
                "    * \n",
                "   ** \n",
                "      "
            );

            let end = concat!(
                "      \n",
                " **   \n",
                " **   \n",
                "   ** \n",
                "   ** \n",
                "      "
            );

            {
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(initial).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(end, renderer.print_grid());
        }
    }

    // These tests test travelling spaceships.
    mod spaceships {
        use super::*;
        use crate::game::UserCellGenerator;

        #[test]
        fn glider() {
            let mut renderer = MockRenderer::new_with_size(Dimensions { width: 6, height: 6 });

            // Initial board state.
            let phase_1 = concat!(
                "      \n",
                "  *   \n",
                "   ** \n",
                "  **  \n",
                "      \n",
                "      "
            );

            // Second board state
            let phase_2 = concat!(
                "      \n",
                "   *  \n",
                "    * \n",
                "  *** \n",
                "      \n",
                "      "
            );

            {
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(phase_1).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(phase_2, renderer.print_grid());

            // Third board state
            let phase_3 = concat!(
                "      \n",
                "      \n",
                "  * * \n",
                "   ** \n",
                "   *  \n",
                "      "
            );

            {
                // Create a new game board to calculate from phase 2 to phase 3.
                // In reality we would simply reuse the existing game board.
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(phase_2).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(phase_3, renderer.print_grid());

            // Third board state
            let phase_4 = concat!(
                "      \n",
                "      \n",
                "    * \n",
                "  * * \n",
                "   ** \n",
                "      "
            );

            {
                // Create a new game board to calculate from phase 2 to phase 3.
                // In reality we would simply reuse the existing game board.
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(phase_3).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(phase_4, renderer.print_grid());

            // Now we should return to phase 1. Kinda. The phase 1 shape will
            // be the same, but it will have translated down and to the right.
            let phase_1_translated = concat!(
                "      \n",
                "      \n",
                "   *  \n",
                "    **\n",
                "   ** \n",
                "      "
            );

            {
                // Create a new game board to calculate from phase 2 to phase 3.
                // In reality we would simply reuse the existing game board.
                let mut game_board = GameBoard::new_from_seed(
                    renderer.get_grid_size(),
                    UserCellGenerator::from_str(phase_4).unwrap(),
                    &mut renderer
                );

                game_board.calculate_iteration();
            }

            assert_eq!(phase_1_translated, renderer.print_grid());
        }
    }
}
