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

use crate::game::Cell;
use xy_utils::{Dimensions, Point};

/// This traits class defines the required interface for a game board renderer.
///
/// The renderer can be initialized to draw the initial user interface, then can
/// be updated using the `apply_changes()` method. The cell changes will then
/// be emitted by the renderer.
///
/// Renderers are expected to handle their own errors. A renderer that cannot
/// complete an operation are expected to handle the error internally so that
/// the renderer can correct when the next changes are applied. As a
/// consequence, mutable operations do NOT return errors.
pub trait Renderer {
    /// Initialize the rendered user interface.
    fn initialize(&mut self);

    /// Get the size of the renderable grid.
    fn get_grid_size(&self) -> Dimensions;

    /// Render cell changes. Accepts a list of cell changes. The renderer can
    /// choose to render these changes however it chooses.
    ///
    /// ## Arguments
    ///
    /// * `changes`: The list of changes made by the game board.
    ///
    /// ## Returns
    ///
    /// A result type. If initialization failed, the string error is returned.
    fn apply_changes(&mut self, changes: Vec<(Point, Cell)>);
}

#[cfg(test)]
pub mod mock {
    pub use super::*;

    const DEFAULT_RENDERER_WIDTH: usize = 5;
    const DEFAULT_RENDERER_HEIGHT: usize = 5;

    /// Mock renderer implementation.
    ///
    /// It does not do any actual rendering, but instead can be introspected to
    /// see if the game board has applied appropriate changes.
    pub struct MockRenderer {
        /// The rendered grid. When changes are applied, this grid is updated
        /// with the updated cell states.
        ///
        /// Grid is row-major format, meaning that each index in the outer Vec
        /// represents one row, and the inner Vec is the value of the individual
        /// cells in that row.
        pub rendered_grid: Vec<Vec<Cell>>,
    }

    impl Renderer for MockRenderer {
        fn initialize(&mut self) {}

        fn get_grid_size(&self) -> Dimensions {
            Dimensions { width: self.rendered_grid[0].len(), height: self.rendered_grid.len() }
        }

        fn apply_changes(&mut self, changes: Vec<(Point, Cell)>) {
            for (cell_address, cell_state) in changes {
                self.rendered_grid[cell_address.y][cell_address.x] = cell_state;
            }
        }
    }

    impl MockRenderer {
        pub fn new() -> MockRenderer {
            Self::new_with_size(Dimensions {
                width: DEFAULT_RENDERER_WIDTH,
                height: DEFAULT_RENDERER_HEIGHT,
            })
        }

        pub fn new_with_size(size: Dimensions) -> MockRenderer {
            // All cells start as dead.
            let mut row_template = vec![];
            row_template.resize(size.width, Cell::Dead);

            let mut grid = vec![];
            grid.resize(size.height, row_template);
            MockRenderer { rendered_grid: grid }
        }

        // Helper function that prints the grid as a string for easy test views.
        pub fn print_grid(&self) -> String {
            let mut row_str = String::new();
            let mut result = String::new();
            let mut is_first_iteration = true;

            for row in &self.rendered_grid {
                if is_first_iteration {
                    is_first_iteration = false;
                } else {
                    row_str.clear();
                    row_str.push('\n');
                }
                for cell in row {
                    match cell {
                        Cell::Alive => row_str.push('*'),
                        Cell::Dead => row_str.push(' '),
                    };
                }

                result.push_str(&row_str);
            }

            // Remove the final newline from the string
            result
        }
    }
}
