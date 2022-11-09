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

use clap::Parser;
use xy_utils::Dimensions;

/// Command line arguments.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Frequency in which the game board is updated.
    #[arg(short, long, value_name = "HZ", default_value_t = 4)]
    pub update_frequency: u32,

    /// Size of the grid that the game will be played in.
    #[arg(
        short = 's',
        long,
        value_name = "HxW",
        default_value_t = Dimensions::create_empty()
    )]
    pub grid_size: Dimensions,

    /// File to read to seed the game board.
    #[arg(
        short = 'f',
        long,
        default_value_t = String::new()
    )]
    pub game_board_file: String,
}
