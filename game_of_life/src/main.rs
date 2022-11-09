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

use std::{fs::File, io::Read, path::Path, thread, time::Duration};

use clap::Parser;
use crossterm::event;
use rand;

use game::{CellGenerator, GameBoard, RandomCellGenerator, Renderer, UserCellGenerator};
use tui::DefaultPlotter;
use tui_renderer::TuiRenderer;
use xy_utils::Dimensions;

mod cli;
mod game;
mod tui_renderer;

fn get_game_board_seed_from_file(file_path_str: &str) -> String {
    if file_path_str.is_empty() {
        String::new()
    } else {
        let path = Path::new(file_path_str);

        let mut file = File::open(path).unwrap();
        let mut output = String::new();
        file.read_to_string(&mut output).unwrap();
        output
    }
}

fn create_game_board<CellGeneratorT: CellGenerator, RendererT: Renderer>(
    cell_generator: CellGeneratorT,
    size: Dimensions,
    renderer: &mut RendererT,
) -> GameBoard<RendererT> {
    GameBoard::new_from_seed(size, cell_generator, renderer)
}

fn calculate_game_board_size(
    user_grid_size: Dimensions,
    renderer_grid_size: Dimensions,
) -> Dimensions {
    // The user may not provide the dimensions of the game board, so in that
    // case we will use the same dimensions as the UI's game grid.
    let height = if user_grid_size.is_height_defined() {
        user_grid_size.height
    } else {
        renderer_grid_size.height
    };

    let width = if user_grid_size.is_width_defined() {
        user_grid_size.width
    } else {
        renderer_grid_size.width
    };

    Dimensions { width, height }
}

fn main() {
    let args = cli::Args::parse();

    // Set up the TUI graphics renderer.
    let mut renderer = TuiRenderer::new(DefaultPlotter::create_from_stdout(), args.grid_size);
    renderer.initialize();

    // If the user has provided their own game seed, we should try to use it.
    let game_board_seed = get_game_board_seed_from_file(&args.game_board_file);
    let game_board_size = calculate_game_board_size(args.grid_size, renderer.get_grid_size());

    let mut game_board = if game_board_seed.is_empty() {
        create_game_board(
            RandomCellGenerator { rng: rand::thread_rng() },
            game_board_size,
            &mut renderer,
        )
    } else {
        create_game_board(
            UserCellGenerator::from_str(&game_board_seed).unwrap(),
            game_board_size,
            &mut renderer,
        )
    };

    let nanos_per_iteration = ((1.0 / args.update_frequency as f64) * 1000000000.0) as u32;
    let mut exiting = false;
    let ctrl_c_keyevent =
        event::KeyEvent::new(event::KeyCode::Char('c'), event::KeyModifiers::CONTROL);

    while !exiting {
        game_board.calculate_iteration();
        thread::sleep(Duration::new(0, nanos_per_iteration));

        if event::poll(Duration::from_secs(0)).unwrap() {
            // User made a keypress, check if they CTRL+C'd...
            match event::read().unwrap() {
                event::Event::Key(key_event) => exiting = key_event == ctrl_c_keyevent,
                _ => {}
            }
        }
    }
}
