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

use unicode_segmentation::UnicodeSegmentation;

use crate::lowlevel::*;
use xy_utils::{Dimensions, Point};

const EMPTY_CHAR: char = ' ';

/// Commands that the Canvas will execute when rendered.
#[derive(Clone)]
enum CanvasCommand {
    /// Draw a string of zero or more characters to the terminal.
    Draw(Point, String),

    /// Change the drawing color for the next commands.
    ChangeColor(Paintbrush),
}

/// Struct that defines a simple rectangular grid of characters.
pub struct Canvas {
    /// The location in the UI to render the canvas.
    pub position: Point,

    /// The size of the canvas.
    pub size: Dimensions,

    /// The label text (updated via the update() method).
    changes: Vec<CanvasCommand>,
}

impl Canvas {
    pub fn new(position: Point, size: Dimensions) -> Self {
        let mut result = Canvas { position, size, changes: vec![] };
        let size = &result.size;

        // Set up the initial grid.
        let mut columns_str = String::with_capacity(size.width);
        for _ in 0..size.width {
            columns_str.push(EMPTY_CHAR);
        }

        // Clear the canvas the first time it is rendered.
        let mut row_idx = 0;
        result.changes.resize_with(result.size.height as usize, || {
            row_idx += 1;
            CanvasCommand::Draw(Point { x: 0, y: row_idx - 1 }, columns_str.clone())
        });

        result
    }

    /// Draw one of more characters onto the Canvas.
    ///
    /// Call render() to update the canvas in the UI.
    ///
    /// ## Arguments
    ///
    /// * `paintbrush`: Color information to draw the characters with.
    /// * `point`: The location to set in the canvas. This is relative to the
    ///   top-left corner of the canvas, not the whole terminal. So
    ///   `Point {x: 0, y:0}` would be the first cell of the Canvas in the top-
    ///   left corner.
    /// * `val`: The value to set to. The val should be zero or more unicode
    ///   characters. If the number of characters goes beyond the width of the
    ///   canvas, an error is returned.
    pub fn draw_str(
        &mut self,
        paintbrush: Paintbrush,
        position: Point,
        val: &str,
    ) -> Result<(), std::io::Error> {
        // Prerequisite check, ensure that the val only has one grapheme.
        if (val.graphemes(true).count() + position.x) > self.size.width {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "String of characters would exceed the width of the canvas",
            ));
        }
        self.changes.push(CanvasCommand::ChangeColor(paintbrush));
        self.changes.push(CanvasCommand::Draw(position, val.to_string()));
        Ok(())
    }

    /// Render the text label using the provided low level UI plotter.
    pub fn render<PlotterT: Plotter>(
        &mut self,
        plotter: &mut PlotterT,
    ) -> Result<(), std::io::Error> {
        for command in self.changes.drain(..) {
            match command {
                CanvasCommand::Draw(pos, s) => plotter
                    .plot(Point { x: pos.x + self.position.x, y: pos.y + self.position.y }, s)?,
                CanvasCommand::ChangeColor(pb) => plotter.set_paintbrush(&pb)?,
            };
        }

        Ok(())
    }
}

#[cfg(test)]
mod canvas_tests {
    use super::*;

    #[test]
    fn has_a_constructor() {
        let canvas = Canvas::new(Point { x: 1, y: 2 }, Dimensions { width: 3, height: 3 });

        assert_eq!(Point { x: 1, y: 2 }, canvas.position);
        assert_eq!(Dimensions { width: 3, height: 3 }, canvas.size);
    }

    #[test]
    fn clears_all_rows_on_init() {
        let mut plotter = mock::MockPlotter::new();
        let mut canvas = Canvas::new(Point { x: 1, y: 2 }, Dimensions { width: 3, height: 3 });

        canvas.render(&mut plotter).unwrap();

        assert_eq!(3, plotter.command_list.len());

        match &plotter.command_list[0] {
            mock::MockPlotterCommand::PlotObject(point, s) => {
                assert_eq!(point.x, 1);
                assert_eq!(point.y, 2);
                assert_eq!(s, "   ");
            }
            _ => panic!("Incorrect first plotter command"),
        }
        match &plotter.command_list[1] {
            mock::MockPlotterCommand::PlotObject(point, s) => {
                assert_eq!(point.x, 1);
                assert_eq!(point.y, 3);
                assert_eq!(s, "   ");
            }
            _ => panic!("Incorrect first plotter command"),
        }
        match &plotter.command_list[2] {
            mock::MockPlotterCommand::PlotObject(point, s) => {
                assert_eq!(point.x, 1);
                assert_eq!(point.y, 4);
                assert_eq!(s, "   ");
            }
            _ => panic!("Incorrect first plotter command"),
        }
    }

    #[test]
    fn can_be_rendered_with_a_lowlevel_plotter() {
        let mut plotter = mock::MockPlotter::new();
        let mut canvas = Canvas::new(Point { x: 1, y: 2 }, Dimensions { width: 3, height: 3 });

        canvas.draw_str(Paintbrush::create_default(), Point { x: 0, y: 0 }, "***").unwrap();
        canvas.render(&mut plotter).unwrap();

        plotter.flush().unwrap();

        // Plotter should have received several commands here:
        // 1. Clear the canvas (3 commands)
        // 2. Set the style options for the output.
        // 3. Plot the label.
        // 4. Flush the commands to the "output terminal" which is faked out.
        assert_eq!(6, plotter.command_list.len());

        match &plotter.command_list[3] {
            mock::MockPlotterCommand::SetPaintbrush(pb) => {
                assert_eq!(pb.fg, Paintbrush::create_default().fg);
                assert_eq!(pb.bg, Paintbrush::create_default().bg);
                assert_eq!(pb.bold, Paintbrush::create_default().bold);
            }
            _ => panic!("Incorrect first plotter command"),
        }

        // Note here that the point that gets rendered is offset by the canvas'
        // position in the UI. So all coordinates are adjusted by 1 on the
        // X-axis and 2 on the Y=axis.
        match &plotter.command_list[4] {
            mock::MockPlotterCommand::PlotObject(point, s) => {
                assert_eq!(point.x, 1);
                assert_eq!(point.y, 2);
                assert_eq!(s, "***");
            }
            _ => panic!("Incorrect second plotter command"),
        }
        match &plotter.command_list[5] {
            mock::MockPlotterCommand::Flush => {}
            _ => panic!("Incorrect third plotter command"),
        }
    }

    #[test]
    fn will_not_allow_writing_data_out_of_bounds() {
        // In this example, the user attempts to write a string to the canvas
        // that would overflow the edge of the cavas. In this case, the canvas
        // will reject the request and return an error.
        let mut plotter = mock::MockPlotter::new();
        let mut canvas = Canvas::new(Point { x: 1, y: 2 }, Dimensions { width: 3, height: 3 });

        // Even though the canvas could technically print the whole string on
        // a single line, the X offset we provide pushes it beyond the edge.
        if let Ok(_) = canvas.draw_str(Paintbrush::create_default(), Point { x: 1, y: 0 }, "***") {
            panic!("This test should have failed due to writing out of bounds!");
        }

        canvas.render(&mut plotter).unwrap();
        plotter.flush().unwrap();

        // Only 4 command should have been send from the plotter. The initial
        // canvas clearing (x3), then the flush command we just executed above.
        assert_eq!(4, plotter.command_list.len());
    }
}
