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

/// Struct that defines a simple text label rendered to the terminal UI.
pub struct TextLabel {
    /// The paintbrush that sets the color info for the label content.
    paintbrush: Paintbrush,

    /// The location in the UI to render the label.
    position: Point,

    /// The allowed size of the label.
    size: Dimensions,

    /// The label text (updated via the update() method).
    text: String,

    /// Internally calculated character string rows for the label.
    output_text_rows: Vec<String>,
}

impl TextLabel {
    pub fn new(paintbrush: Paintbrush, position: Point, size: Dimensions, text: &str) -> Self {
        let mut result = TextLabel {
            paintbrush,
            position,
            size,
            text: text.to_string(),
            output_text_rows: vec![],
        };

        // Create the renderable character rows.
        result.get_label_output_text();

        result
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    /// Set the paintbrush for this label.
    ///
    /// This allows the format of the label text to be changed as
    /// needed.
    ///
    /// ## Arguments
    ///
    /// * `paintbrush`: the next paintbrush.
    pub fn set_paintbrush(&mut self, paintbrush: Paintbrush) {
        self.paintbrush = paintbrush;
    }

    /// Update the text label.
    ///
    /// Call render() to emit the label to the UI.
    ///
    /// ## Arguments
    ///
    /// * `text`: New text to insert into the label.
    pub fn update(&mut self, text: &str) {
        self.text = text.to_string();
        self.get_label_output_text();
    }

    /// Render the text label using the provided low level UI plotter.
    pub fn render<PlotterT: Plotter>(&self, plotter: &mut PlotterT) -> Result<(), std::io::Error> {
        // We may need to truncate the label text to fit inside the
        plotter.set_paintbrush(&self.paintbrush)?;

        for (i, row) in self.output_text_rows.iter().enumerate() {
            // We need to pad the label with whitespace to overwrite any old
            // changes.
            plotter.plot(Point { x: self.position.x, y: self.position.y + i }, row)?;
        }
        Ok(())
    }

    fn get_label_output_text(&mut self) {
        let max_cols = self.size.width;
        let max_rows = self.size.height;
        let max_text_length = (max_cols * max_rows) as usize;

        // Get the rendered text as a single string. This will let us slice it
        // properly.
        let mut output_text = String::new();
        if self.text.len() <= max_text_length {
            output_text = self.text.clone();
        } else if max_text_length < 3 {
            for _ in 0..max_text_length {
                output_text.push('.');
            }
        } else {
            let graphemes_list =
                &self.text.graphemes(true).collect::<Vec<&str>>()[0..max_text_length - 3];
            for grapheme in graphemes_list {
                output_text += grapheme;
            }
            // Add an ellipsis to indicate truncation.
            output_text += "...";
        }

        // Break the text across multiple lines in case the label is multiple
        // Characters high.
        self.output_text_rows.clear();
        let mut x = 0;
        let mut row_start_byte = 0;
        let mut row_end_byte = 0;
        for grapheme in output_text.graphemes(true) {
            row_end_byte += grapheme.as_bytes().len();
            x += 1;

            if x == max_cols {
                self.output_text_rows.push(output_text[row_start_byte..row_end_byte].to_string());
                x = 0;
                row_start_byte = row_end_byte;
            }
        }

        if row_end_byte != row_start_byte {
            // Make sure to pad the row to ensure that any existing content is
            // erased.
            let mut row = output_text[row_start_byte..row_end_byte].to_string();
            let padding_size = self.size.width - row.len();
            for _ in 0..padding_size {
                row.push_str(" ");
            }
            self.output_text_rows.push(row);
        }

        // We might need to add empty rows to ensure that we erase any existing
        // content.
        if self.output_text_rows.len() < self.size.height {
            let mut empty_row = String::with_capacity(self.size.height);
            for _ in 0..self.size.width {
                empty_row.push(' ');
            }
            while self.output_text_rows.len() < self.size.height {
                self.output_text_rows.push(empty_row.clone());
            }
        }
    }
}

#[cfg(test)]
mod text_label_tests {
    use super::*;

    #[test]
    fn has_a_constructor() {
        let label = TextLabel::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 3, height: 1 },
            "FOO",
        );

        assert_eq!(label.text, "FOO");
    }

    #[test]
    fn can_be_rendered_with_a_lowlevel_plotter() {
        let mut plotter = mock::MockPlotter::new();
        let label = TextLabel::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 3, height: 1 },
            "FOO",
        );

        label.render(&mut plotter).unwrap();
        plotter.flush().unwrap();

        // Plotter should have received several commands here:
        // 1. Set the style options for the output.
        // 2. Plot the label.
        // 3. Flush the commands to the "output terminal" which is faked out.
        assert_eq!(plotter.command_list.len(), 3);

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
                assert_eq!(s, "FOO");
            }
            _ => panic!("Incorrect second plotter command"),
        }

        match &plotter.command_list[2] {
            mock::MockPlotterCommand::Flush => {}
            _ => panic!("Incorrect second plotter command"),
        }
    }

    #[test]
    fn will_not_exceed_its_width() {
        // In the event of label text containing more unicode graphemes than
        // there are free cells in the label, trim the text content and add an
        // ellipsis ("...").
        let mut plotter = mock::MockPlotter::new();
        let label = TextLabel::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 6, height: 1 },
            "FOOBARBAZ",
        );

        label.render(&mut plotter).unwrap();
        plotter.flush().unwrap();

        assert_eq!(plotter.command_list.len(), 3);
        match &plotter.command_list[1] {
            mock::MockPlotterCommand::PlotObject(_, s) => {
                // Desired text should have been shortened.
                assert_eq!(s, "FOO...");
            }
            _ => panic!("Incorrect second plotter command"),
        }
    }

    #[test]
    fn will_not_exceed_its_size_even_if_shorter_than_ellipsis() {
        // The test above asserts that the label will not exceed it's length.
        // If the label cannot even fit an ellipsis, it should just show _n_
        // dots, where _n_ is the number of characters that can be written in
        // the label.
        let mut plotter = mock::MockPlotter::new();
        let label = TextLabel::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 2, height: 1 },
            "FOOBARBAZ",
        );

        label.render(&mut plotter).unwrap();
        plotter.flush().unwrap();

        assert_eq!(plotter.command_list.len(), 3);
        match &plotter.command_list[1] {
            mock::MockPlotterCommand::PlotObject(_, s) => {
                // Desired text should have been shortened.
                assert_eq!(s, "..");
            }
            _ => panic!("Incorrect second plotter command"),
        }
    }

    #[test]
    fn can_be_updated() {
        let mut plotter = mock::MockPlotter::new();
        let mut label = TextLabel::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 6, height: 1 },
            "FOOBAR",
        );

        label.render(&mut plotter).unwrap();
        plotter.flush().unwrap();

        assert_eq!(plotter.command_list.len(), 3);
        match &plotter.command_list[1] {
            mock::MockPlotterCommand::PlotObject(_, s) => {
                assert_eq!(s, "FOOBAR");
            }
            _ => panic!("Incorrect second plotter command"),
        }

        label.update("QUXBARFOO");
        label.render(&mut plotter).unwrap();
        plotter.flush().unwrap();

        assert_eq!(plotter.command_list.len(), 6);
        match &plotter.command_list[4] {
            mock::MockPlotterCommand::PlotObject(_, s) => {
                // Updated text should have been shortened.
                assert_eq!(s, "QUX...");
            }
            _ => panic!("Incorrect second plotter command"),
        }
    }

    #[test]
    fn support_multi_lines() {
        let mut plotter = mock::MockPlotter::new();
        let label = TextLabel::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 3, height: 3 },
            "FOOBARBAZ",
        );

        label.render(&mut plotter).unwrap();
        plotter.flush().unwrap();

        // We should have received three plot commands, one for each line.
        assert_eq!(plotter.command_list.len(), 5);
        if let mock::MockPlotterCommand::PlotObject(_, s) = &plotter.command_list[1] {
            assert_eq!(s, "FOO");
        } else {
            panic!("Incorrect second plotter command");
        }
        if let mock::MockPlotterCommand::PlotObject(_, s) = &plotter.command_list[2] {
            assert_eq!(s, "BAR");
        } else {
            panic!("Incorrect third plotter command");
        }

        if let mock::MockPlotterCommand::PlotObject(_, s) = &plotter.command_list[3] {
            assert_eq!(s, "BAZ");
        } else {
            panic!("Incorrect fourth plotter command");
        }
    }

    #[test]
    fn trimming_works_even_on_multi_line_labels() {
        let mut plotter = mock::MockPlotter::new();
        let label = TextLabel::new(
            Paintbrush::create_default(),
            Point { x: 1, y: 2 },
            Dimensions { width: 3, height: 2 },
            "FOOBARBAZ",
        );

        label.render(&mut plotter).unwrap();
        plotter.flush().unwrap();

        // We should have received three plot commands, one for each line.
        assert_eq!(plotter.command_list.len(), 4);
        if let mock::MockPlotterCommand::PlotObject(_, s) = &plotter.command_list[2] {
            assert_eq!(s, "...");
        } else {
            panic!("Incorrect third plotter command");
        }
    }
}
