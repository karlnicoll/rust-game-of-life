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

use crate::components::TextLabel;
use crate::lowlevel::*;
use xy_utils::{Dimensions, Point};

/// Struct that renders a label that holds a name and a numeric value.
///
/// The count is rendered as a label in two parts, a "Key" which describes the
/// count, and a value that can be incremented, decremented, or simply set.
///
/// ### Example
///
/// ```
/// use tui::{mock, Paintbrush, Plotter};
/// use tui::components::Count;
/// use xy_utils::{Dimensions, Point};
/// let mut plotter = mock::MockPlotter::new();
/// let mut label = Count::new(
///     Paintbrush::create_default(),
///     Point { x: 1, y: 2 },
///     Dimensions { width: 16, height: 1 },
///     10,
///     "Text label",
///     false
/// );
///
/// label.render(&mut plotter).unwrap();
/// plotter.flush().unwrap();
///
/// // Renders label as "Text label: 0   " (extra spaces are padding)
/// ```
pub struct Count {
    key: TextLabel,
    value_label: TextLabel,
    last_value: usize,
    value: usize,
    color_code_value: bool,
}

impl Count {
    /// Create a new Count component.
    ///
    /// ## Arguments
    ///
    /// * `paintbrush`: the paintbrush to use for the text.
    /// * `position`: the location of the component.
    /// * `size`: total allowed size of the label.
    /// * `key_width`: the number of horizontal cells to reserve for the key
    ///   text. When rendered, this width will be two characters larger to
    ///   account for the ": " appended to the key text.
    /// * `key_text`: the text that is prefixed to the count, to give it an
    ///   identity.
    /// * `color_code`: when TRUE, the value will be made red when the count
    ///   goes down, or green when it goes up.
    pub fn new(
        paintbrush: Paintbrush,
        position: Point,
        size: Dimensions,
        key_width: usize,
        key_text: &str,
        color_code: bool,
    ) -> Self {
        let actual_key_width = key_width + 2;
        let key_text = format!("{}: ", key_text);
        let key = TextLabel::new(
            paintbrush.clone(),
            position,
            Dimensions { width: actual_key_width, ..size },
            &key_text,
        );

        let value_label = TextLabel::new(
            paintbrush,
            Point { x: position.x + actual_key_width, ..position },
            Dimensions { width: size.width - actual_key_width, ..size },
            "0",
        );

        Count { key, value_label, last_value: 0, value: 0, color_code_value: color_code }
    }

    /// Update the count.
    ///
    /// Call render() to emit the label to the UI.
    ///
    /// ## Arguments
    ///
    /// * `new_value`: New value to give the count.
    pub fn update(&mut self, new_value: usize) {
        self.value = new_value;
        self.value_label.update(&self.value.to_string());
    }

    /// Increment the count by 1.
    pub fn increment(&mut self) {
        self.update(self.value + 1);
    }

    /// Decrement the count by 1.
    pub fn decrement(&mut self) {
        self.update(self.value - 1);
    }

    /// Render the text label using the provided low level UI plotter.
    pub fn render<PlotterT: Plotter>(
        &mut self,
        plotter: &mut PlotterT,
    ) -> Result<(), std::io::Error> {
        self.key.render(plotter)?;

        if self.color_code_value {
            let new_paintbrush = if self.value > self.last_value {
                Paintbrush { fg: Color::Green, ..Paintbrush::create_default() }
            } else if self.value < self.last_value {
                Paintbrush { fg: Color::Red, ..Paintbrush::create_default() }
            } else {
                Paintbrush::create_default()
            };
            self.value_label.set_paintbrush(new_paintbrush);
        }
        self.last_value = self.value;
        self.value_label.render(plotter)
    }
}

#[cfg(test)]
mod count_tests {
    use super::*;

    #[test]
    fn has_a_constructor() {
        let _ = Count::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 10, height: 1 },
            3,
            "FOO",
            false,
        );
    }

    #[test]
    fn displays_the_key_and_value() {
        let mut plotter = mock::MockPlotter::new();
        let mut count = Count::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 10, height: 1 },
            3,
            "FOO",
            false,
        );

        count.render(&mut plotter).unwrap();
        plotter.flush().unwrap();

        // Plotter renders two labels here:
        //
        // * First label is rendered with the key text (e.g. "FOO: ")
        // * Second label is rendered with the value (e.g. 0)
        //
        // Note that the key text is slightly longer than provided in the
        // constructor, since a colon is appended to the end.
        assert_eq!(plotter.command_list.len(), 5);

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
                assert_eq!(s, "FOO: ");
            }
            _ => panic!("Incorrect second plotter command"),
        }

        match &plotter.command_list[2] {
            mock::MockPlotterCommand::SetPaintbrush(pb) => {
                assert_eq!(pb.fg, Paintbrush::create_default().fg);
                assert_eq!(pb.bg, Paintbrush::create_default().bg);
                assert_eq!(pb.bold, Paintbrush::create_default().bold);
            }
            _ => panic!("Incorrect first plotter command"),
        }

        match &plotter.command_list[3] {
            mock::MockPlotterCommand::PlotObject(point, s) => {
                assert_eq!(point.x, 6);
                assert_eq!(point.y, 2);

                // Output is padded with whitespace so that any content is
                // erased when the number gets smaller. For example, if the
                // number from from "10" to "9", the label should not show
                // "90" because the zero was not overwritten.
                assert_eq!(s, "0    ");
            }
            _ => panic!("Incorrect second plotter command"),
        }

        match &plotter.command_list[4] {
            mock::MockPlotterCommand::Flush => {}
            _ => panic!("Incorrect second plotter command"),
        }
    }

    #[test]
    fn can_color_code_the_value() {
        let mut plotter = mock::MockPlotter::new();
        let mut count = Count::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 10, height: 1 },
            3,
            "FOO",
            true,
        );

        count.render(&mut plotter).unwrap();
        count.update(10);
        count.render(&mut plotter).unwrap();
        count.decrement();
        count.render(&mut plotter).unwrap();

        plotter.flush().unwrap();

        // Multiple renders here that should have different results:
        //
        // 1. First render shows a normal label with value zero.
        // 2. Second render shows the value has increased, so the value is
        //    green.
        // 3. Third render shows the value has decreased, so the value is
        //    red.
        //
        // Each render (as we see from the previous test) triggers four plotter
        // commands. Thirteen total since we also flush the plotter, which is
        // also a command.
        assert_eq!(plotter.command_list.len(), 13);

        match &plotter.command_list[2] {
            mock::MockPlotterCommand::SetPaintbrush(pb) => {
                assert_eq!(pb.fg, Paintbrush::create_default().fg);
                assert_eq!(pb.bg, Paintbrush::create_default().bg);
                assert_eq!(pb.bold, Paintbrush::create_default().bold);
            }
            _ => panic!("Incorrect first plotter command"),
        }

        match &plotter.command_list[3] {
            mock::MockPlotterCommand::PlotObject(_, s) => {
                assert_eq!(s, "0    ");
            }
            _ => panic!("Incorrect second plotter command"),
        }

        match &plotter.command_list[6] {
            mock::MockPlotterCommand::SetPaintbrush(pb) => {
                assert_eq!(pb.fg, Color::Green);
                assert_eq!(pb.bg, Paintbrush::create_default().bg);
            }
            _ => panic!("Incorrect first plotter command"),
        }

        match &plotter.command_list[7] {
            mock::MockPlotterCommand::PlotObject(_, s) => {
                assert_eq!(s, "10   ");
            }
            _ => panic!("Incorrect second plotter command"),
        }

        match &plotter.command_list[10] {
            mock::MockPlotterCommand::SetPaintbrush(pb) => {
                assert_eq!(pb.fg, Color::Red);
                assert_eq!(pb.bg, Paintbrush::create_default().bg);
            }
            _ => panic!("Incorrect first plotter command"),
        }

        match &plotter.command_list[11] {
            mock::MockPlotterCommand::PlotObject(_, s) => {
                assert_eq!(s, "9    ");
            }
            _ => panic!("Incorrect second plotter command"),
        }

        match &plotter.command_list[12] {
            mock::MockPlotterCommand::Flush => {}
            _ => panic!("Incorrect second plotter command"),
        }
    }
}
