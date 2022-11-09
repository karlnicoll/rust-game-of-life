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

use crate::game::{Cell, Renderer};
use tui::components::{Border, Canvas, Count, TextLabel};
use tui::{Color, Paintbrush, Plotter};
use xy_utils::{Dimensions, Point};

/// Renderer implementation that renders the game board to a terminal user
/// interface.
pub struct TuiRenderer<PlotterT: Plotter> {
    plotter: PlotterT,

    // Need to keep track of the current cell states internally for rendering
    // purposes.
    current_cell_states: Vec<&'static str>,

    // Message field, provides any informational stuff about errors etc.
    message_field: (TextLabel, TextLabel),

    // Game canvas. Where the game of life is rendered.
    game_area: (Border, Canvas),

    // Game stats.
    population_field: Count,
    generation_field: Count,
    total_births_field: Count,
    total_deaths_field: Count,
}

impl<PlotterT: Plotter> TuiRenderer<PlotterT> {
    /// Create a new renderer instance.
    ///
    /// ## Arguments
    ///
    /// * `plotter`: The plotter object that will be used to create the
    ///   rendered user interface.
    /// * `game_size`: The grid dimensions to use for the actual game.
    pub fn new(plotter: PlotterT, game_size: Dimensions) -> Self {
        let ui_size = plotter.get_plot_area();

        // Some of the area needs to be reserved for the labels.
        let num_top_labels_rows = 1; // Messages are on the top row.
        let num_bottom_labels_rows = 2; // Stats take up the bottom two rows.
        let border_total_size = 2; // Two chars required for border (one on each opposing side).
        let total_reserved_rows = num_top_labels_rows + num_bottom_labels_rows + border_total_size;
        let total_reserved_columns = border_total_size;
        let game_dimensions = Self::create_game_dimensions(
            &game_size,
            &ui_size,
            total_reserved_rows,
            total_reserved_columns,
        );

        let mut initial_cell_states = Vec::with_capacity(game_dimensions.total_area());
        initial_cell_states.resize(initial_cell_states.capacity(), " ");

        Self {
            plotter,
            current_cell_states: initial_cell_states,
            message_field: Self::create_message_field(ui_size.width),
            game_area: Self::create_game_area(Point { x: 0, y: 1 }, game_dimensions),
            population_field: Self::create_stats_field(
                Point { x: 0, y: ui_size.height - 2 },
                Dimensions { height: 1, width: ui_size.width / 2 },
                "Population",
                true,
            ),
            generation_field: Self::create_stats_field(
                Point { x: ui_size.width / 2, y: ui_size.height - 2 },
                Dimensions { height: 1, width: ui_size.width / 2 },
                "Generation",
                false,
            ),
            total_births_field: Self::create_stats_field(
                Point { x: 0, y: ui_size.height - 1 },
                Dimensions { height: 1, width: ui_size.width / 2 },
                "Births",
                false,
            ),
            total_deaths_field: Self::create_stats_field(
                Point { x: ui_size.width / 2, y: ui_size.height - 1 },
                Dimensions { height: 1, width: ui_size.width / 2 },
                "Deaths",
                false,
            ),
        }
    }

    pub fn print_message(&mut self, message: &str) {
        self.message_field.1.update(message);
        self.message_field.1.render(&mut self.plotter).unwrap();
    }

    fn create_game_dimensions(
        game_area: &Dimensions,
        ui_size: &Dimensions,
        reserved_rows: usize,
        reserved_columns: usize,
    ) -> Dimensions {
        // the game area height should be halved if provided by the user, since
        // we can fit two blocks per character on the Y axis (e.g. ▀ and ▄)
        let actual_game_area_height = game_area.height / 2;

        Dimensions {
            height: Self::calculate_optimal_game_area_dimension(
                actual_game_area_height,
                ui_size.height - reserved_rows,
            ),
            width: Self::calculate_optimal_game_area_dimension(
                game_area.width,
                ui_size.width - reserved_columns,
            ),
        }
    }

    fn calculate_optimal_game_area_dimension(game_dimension: usize, ui_dimension: usize) -> usize {
        // If the game dimension is unknown, or the game dimension is bigger
        // than the UI area, clamp the canvas area to the available
        // height/width.
        if game_dimension == 0 || game_dimension > ui_dimension {
            ui_dimension
        } else {
            game_dimension
        }
    }

    fn create_message_field(total_width: usize) -> (TextLabel, TextLabel) {
        let default_paintbrush = Paintbrush::create_default();
        (
            TextLabel::new(
                default_paintbrush.clone(),
                Point { x: 0, y: 0 },
                Dimensions { width: 9, height: 1 },
                "Messages:",
            ),
            TextLabel::new(
                default_paintbrush,
                Point { x: 10, y: 0 },
                Dimensions { width: total_width - 10, height: 1 },
                "",
            ),
        )
    }

    fn create_game_area(position: Point, game_playable_area: Dimensions) -> (Border, Canvas) {
        // This function assumes that the border has already been subtracted.
        let game_board_size =
            Dimensions { height: game_playable_area.height, width: game_playable_area.width };
        let game_board_position = Point { x: position.x + 1, y: position.y + 1 };

        (
            Border::new(
                Paintbrush { fg: Color::Cyan, bg: Color::Unset, bold: false },
                position,
                Dimensions {
                    width: game_playable_area.width + 2,
                    height: game_playable_area.height + 2,
                },
            ),
            Canvas::new(game_board_position, game_board_size),
        )
    }

    fn create_stats_field(
        position: Point,
        size: Dimensions,
        key_text: &str,
        color_coded: bool,
    ) -> Count {
        let paintbrush = Paintbrush::create_default();
        const KEY_WIDTH: usize = 12;
        Count::new(paintbrush, position, size, KEY_WIDTH, key_text, color_coded)
    }

    fn set_game_cell(&mut self, cell_address: Point, new_value: Cell) -> Result<(), String> {
        if !self.cell_is_renderable(&cell_address) {
            self.print_message(&format!(
                "Ignored cell outside of printable area {:?} (max: {}x{})",
                cell_address,
                self.game_area.1.size.width,
                self.game_area.1.size.height * 2
            ));
            return Ok(());
        }

        // Get the char that we should print to the screen. One of :
        //
        // * " " (empty)
        // * "▀"
        // * "█"
        // * "▄"
        let (ui_address, new_ui_value) = self.get_new_ui_value(cell_address, new_value);

        let canvas = &mut self.game_area.1;
        let result = canvas.draw_str(Paintbrush::create_default(), ui_address, new_ui_value);
        if let Err(error) = result {
            Err(error.to_string())
        } else {
            Ok(())
        }
    }

    fn cell_is_renderable(&self, cell_address: &Point) -> bool {
        let canvas = &self.game_area.1;
        // Ignore cells outside the renderable area.
        let max_x_address = canvas.size.width;
        let max_y_address = canvas.size.height * 2; // Two cells per TUI character on the Y axis.

        (cell_address.x < max_x_address) && (cell_address.y < max_y_address)
    }

    fn get_new_ui_value(&mut self, cell_address: Point, new_value: Cell) -> (Point, &'static str) {
        let ui_point = Point { x: cell_address.x, y: cell_address.y / 2 };

        let ui_point_index = (ui_point.y * self.game_area.1.size.width) + ui_point.x;
        let current_ui_value = self.current_cell_states[ui_point_index];
        let is_top_half_of_character = (cell_address.y % 2) == 0;

        let new_char = if is_top_half_of_character {
            match new_value {
                Cell::Alive => {
                    match current_ui_value {
                        "▄" | "█" => "█", // Filling in the full block.
                        _ => "▀",
                    }
                }
                Cell::Dead => match current_ui_value {
                    "▄" | "█" => "▄",
                    _ => " ",
                },
            }
        } else {
            match new_value {
                Cell::Alive => match current_ui_value {
                    "▀" | "█" => "█",
                    _ => "▄",
                },
                Cell::Dead => match current_ui_value {
                    "▀" | "█" => "▀",
                    _ => " ",
                },
            }
        };

        self.current_cell_states[ui_point_index] = new_char;
        (ui_point, new_char)
    }

    fn increase_population(&mut self) {
        self.population_field.increment();
        self.total_births_field.increment();
    }

    fn decrease_population(&mut self) {
        self.population_field.decrement();
        self.total_deaths_field.increment();
    }
}

impl<PlotterT: Plotter> Renderer for TuiRenderer<PlotterT> {
    fn initialize(&mut self) {
        self.print_message("Game board is initialized.");

        self.message_field.0.render(&mut self.plotter).unwrap();
        self.game_area.0.render(&mut self.plotter).unwrap();

        // Render the UI with zero changes initially.
        self.apply_changes(vec![]);
    }

    fn get_grid_size(&self) -> Dimensions {
        Dimensions { width: self.game_area.1.size.width, height: self.game_area.1.size.height * 2 }
    }

    fn apply_changes(&mut self, changes: Vec<(Point, Cell)>) {
        self.generation_field.increment();
        for (cell_address, cell_state) in changes {
            match cell_state {
                Cell::Alive => {
                    self.increase_population();
                }
                Cell::Dead => {
                    self.decrease_population();
                }
            };

            if let Err(error) = self.set_game_cell(cell_address, cell_state) {
                self.print_message(&format!("Error: {}", &error));
            }
        }

        if let Err(error) = self.population_field.render(&mut self.plotter) {
            self.print_message(&format!("Error: {}", error.to_string()));
        }
        if let Err(error) = self.generation_field.render(&mut self.plotter) {
            self.print_message(&format!("Error: {}", error.to_string()));
        }
        if let Err(error) = self.total_births_field.render(&mut self.plotter) {
            self.print_message(&format!("Error: {}", error.to_string()));
        }
        if let Err(error) = self.total_deaths_field.render(&mut self.plotter) {
            self.print_message(&format!("Error: {}", error.to_string()));
        }

        if let Err(error) = self.game_area.1.render(&mut self.plotter) {
            self.print_message(&format!("Error: {}", error.to_string()));
        }

        if let Err(error) = self.plotter.flush() {
            self.print_message(&format!("Error: {}", error.to_string()));
        }
    }
}
