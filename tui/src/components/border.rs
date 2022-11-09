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

use crate::lowlevel::*;
use xy_utils::{Dimensions, Point};

const TOP_LEFT_CORNER: &str = "┌";
const TOP_RIGHT_CORNER: &str = "┐";
const BOTTOM_RIGHT_CORNER: &str = "┘";
const BOTTOM_LEFT_CORNER: &str = "└";
const HORIZONTAL_LINE: &str = "─";
const VERTICAL_LINE: &str = "│";

/// Struct that defines a simple rectangular border that can surround zero or
/// more other components.
///
/// The border does not attach in any meaningful sense to other components. It
/// is extremely simple in that it will render itself once. Other components can
/// place themselves within the border and render themselves without affecting
/// the border.
///
/// Note that since the border has no meaningful boundary enforcement, other
/// components can easily overwrite it.
///
/// ## Example
///
/// ```
/// # use tui::mock::MockPlotter as MyPlotter;
/// use tui::components::Border;
/// use tui::Paintbrush;
/// use xy_utils::{Dimensions, Point};
///
/// let mut plotter = MyPlotter::new();
/// let border = Border::new(
///     Paintbrush::create_default(),
///     Point { x: 0, y: 0 },
///     Dimensions { width: 15, height: 10 }
/// );
///
/// border.render(&mut plotter).unwrap();
/// ```
pub struct Border {
    /// The paintbrush used to draw the border.
    pub paintbrush: Paintbrush,

    /// The location in the UI to render the canvas.
    pub position: Point,

    /// The size of the canvas.
    pub size: Dimensions,
}

impl Border {
    pub fn new(paintbrush: Paintbrush, position: Point, size: Dimensions) -> Self {
        Border { paintbrush, position, size }
    }

    /// Render the text label using the provided low level UI plotter.
    pub fn render<PlotterT: Plotter>(&self, plotter: &mut PlotterT) -> Result<(), std::io::Error> {
        let top_left = self.position;
        let bottom_right = Point {
            x: self.position.x + self.size.width - 1,
            y: self.position.y + self.size.height - 1,
        };
        let bottom_left = Point { x: top_left.x, y: bottom_right.y };
        plotter.set_paintbrush(&self.paintbrush)?;

        // Draw the top row. Pre-allocate double capacity on the assumption that
        // the box drawing characters take two unicode bytes. I don't know how
        // many bytes are actually required and I'm too important to find out.
        let mut row_string = String::with_capacity(self.size.width * 2);
        row_string.push_str(TOP_LEFT_CORNER);
        for _ in 1..self.size.width - 1 {
            row_string.push_str(HORIZONTAL_LINE);
        }
        row_string.push_str(TOP_RIGHT_CORNER);

        plotter.plot(self.position, &row_string)?;

        // Draw vertical lines. Don't draw in the top and bottom row as the top
        // row was already drawn above, and the bottom row will be drawn after
        // this loop.
        for row_idx in (top_left.y + 1)..=(bottom_left.y - 1) {
            // Left side of the border.
            plotter.plot(Point { x: self.position.x, y: row_idx }, VERTICAL_LINE)?;

            // Right side of the border.
            plotter.plot(Point { x: bottom_right.x, y: row_idx }, VERTICAL_LINE)?;
        }

        // Draw the bottom row.
        row_string.clear();
        row_string.push_str(BOTTOM_LEFT_CORNER);
        for _ in 1..self.size.width - 1 {
            row_string.push_str(HORIZONTAL_LINE);
        }
        row_string.push_str(BOTTOM_RIGHT_CORNER);

        plotter.plot(bottom_left, &row_string)?;

        Ok(())
    }
}

#[cfg(test)]
mod border_tests {
    use super::*;

    #[test]
    fn has_a_constructor() {
        let border = Border::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 3, height: 3 },
        );

        assert_eq!(Point { x: 1, y: 2 }, border.position);
        assert_eq!(Dimensions { width: 3, height: 3 }, border.size);
    }

    #[test]
    fn can_be_rendered_with_a_lowlevel_plotter() {
        let mut plotter = mock::MockPlotter::new();
        let border = Border::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 3, height: 3 },
        );

        border.render(&mut plotter).unwrap();

        plotter.flush().unwrap();

        // Plotter should have received several commands here:
        // 1. Clear the canvas (3 commands)
        // 2. Set the style options for the output.
        // 3. Plot the label.
        // 4. Flush the commands to the "output terminal" which is faked out.
        assert_eq!(6, plotter.command_list.len());

        match &plotter.command_list[0] {
            mock::MockPlotterCommand::SetPaintbrush(pb) => {
                assert_eq!(pb.fg, Paintbrush::create_default().fg);
                assert_eq!(pb.bg, Paintbrush::create_default().bg);
                assert_eq!(pb.bold, Paintbrush::create_default().bold);
            }
            _ => panic!("Incorrect first plotter command"),
        }
        match &plotter.command_list[1] {
            mock::MockPlotterCommand::PlotObject(point, s) => {
                assert_eq!(point.x, 1);
                assert_eq!(point.y, 2);
                assert_eq!(s, "┌─┐");
            }
            _ => panic!("Incorrect second plotter command"),
        }
        match &plotter.command_list[2] {
            mock::MockPlotterCommand::PlotObject(point, s) => {
                assert_eq!(point.x, 1);
                assert_eq!(point.y, 3);
                assert_eq!(s, "│");
            }
            _ => panic!("Incorrect third plotter command"),
        }
        match &plotter.command_list[3] {
            mock::MockPlotterCommand::PlotObject(point, s) => {
                assert_eq!(point.x, 3);
                assert_eq!(point.y, 3);
                assert_eq!(s, "│");
            }
            _ => panic!("Incorrect fourth plotter command"),
        }
        match &plotter.command_list[4] {
            mock::MockPlotterCommand::PlotObject(point, s) => {
                assert_eq!(point.x, 1);
                assert_eq!(point.y, 4);
                assert_eq!(s, "└─┘");
            }
            _ => panic!("Incorrect fifth plotter command"),
        }
        match &plotter.command_list[5] {
            mock::MockPlotterCommand::Flush => {}
            _ => panic!("Incorrect fifth plotter command"),
        }
    }
}
