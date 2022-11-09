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

use std::{fmt, str::FromStr};

/// Generic point type.
///
/// Holds X and Y co-ordinates to a point in 2D space. This type does not
/// enforce any units of measurement on the user of this type. The user can
/// define "X" and "Y" to be any unit they want (e.g. pixel, character, etc).
///
/// Defines the x and y position of an item in a grid.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Point {
    pub x: usize,
    pub y: usize,
}

/// The Dimensions struct provides a structure for holding the width and height
/// of the game board. It provides a parser function to allow it to be generated
/// from a string of the format `WxH`.
///
/// ## Example
///
/// ```
/// use xy_utils::Dimensions;
/// use std::str::FromStr;
///
/// let dimensions_str = "5x4";
/// let dimensions = Dimensions::from_str(dimensions_str).unwrap();
///
/// assert_eq!(5, dimensions.width);
/// assert_eq!(4, dimensions.height);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Dimensions {
    pub width: usize,
    pub height: usize,
}

impl Dimensions {
    // Create empty dimensions object. Height and width must be calculated.
    pub fn create_empty() -> Dimensions {
        Dimensions { width: 0, height: 0 }
    }

    /// Calculate the total area of the grid described by this Dimensions
    /// instance.
    ///
    /// ## Example
    ///
    /// ```
    /// use xy_utils::Dimensions;
    /// let d = Dimensions {width: 6, height: 4};
    /// assert_eq!(24, d.total_area());
    /// ```
    pub fn total_area(&self) -> usize {
        self.height as usize * self.width as usize
    }

    /// Is the height of the grid defined?
    pub fn is_height_defined(&self) -> bool {
        self.height != 0
    }

    /// Is the width of the grid defined?
    pub fn is_width_defined(&self) -> bool {
        self.width != 0
    }
}

// Implementing this trait so that CLI can parse Dimensions objects. See:
// https://docs.rs/clap/latest/clap/macro.value_parser.html
impl FromStr for Dimensions {
    type Err = String;

    fn from_str(s: &str) -> Result<Dimensions, String> {
        if s.is_empty() || (s == "calculated") {
            Ok(Dimensions::create_empty())
        } else {
            let parts = s.split('x').collect::<Vec<&str>>();

            if parts.len() != 2 {
                return Err(format!(
                    "Invalid number of dimensions (expected 2, found: {})",
                    parts.len()
                ));
            }

            if let Ok(width) = parts[0].parse::<usize>() {
                if let Ok(height) = parts[1].parse::<usize>() {
                    Ok(Dimensions { height, width })
                } else {
                    Err(format!(
                        "Failed to parse height (expected: unsigned 16-bit integer, received: \"{}\")", parts[1]
                    ))
                }
            } else {
                Err(format!(
                    "Failed to parse width (expected: unsigned 16-bit integer, received: \"{}\")",
                    parts[0]
                ))
            }
        }
    }
}

impl fmt::Display for Dimensions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if (self.height == 0) && (self.width == 0) {
            write!(f, "calculated")
        } else {
            let width_str = stringify_dimension(self.width);
            let height_str = stringify_dimension(self.height);
            write!(f, "{}x{}", width_str, height_str)
        }
    }
}

fn stringify_dimension(dimension: usize) -> String {
    if dimension > 0 {
        format!("{}", dimension)
    } else {
        "[calculated]".to_string()
    }
}

#[cfg(test)]
mod dimensions_tests {
    use super::*;

    #[test]
    fn dimensions_instances_can_be_parsed_from_strings() {
        let dimensions_str = "5x4";
        let dimensions = Dimensions::from_str(dimensions_str).unwrap();
        assert_eq!(5, dimensions.width);
        assert_eq!(4, dimensions.height);
    }

    #[test]
    fn parsing_dimensions_from_string_will_fail_for_1_dimensional_strings() {
        let dimensions_str = "5";
        match Dimensions::from_str(dimensions_str) {
            Err(msg) => assert_eq!(msg, "Invalid number of dimensions (expected 2, found: 1)"),
            _ => {
                panic!("Dimensions::from_str() should have failed for input \"{}\"", dimensions_str)
            }
        }
    }

    #[test]
    fn parsing_dimensions_from_string_will_fail_for_3plus_dimensional_strings() {
        let dimensions_str = "5x10x15";
        match Dimensions::from_str(dimensions_str) {
            Err(msg) => assert_eq!(msg, "Invalid number of dimensions (expected 2, found: 3)"),
            _ => {
                panic!("Dimensions::from_str() should have failed for input \"{}\"", dimensions_str)
            }
        }
    }

    #[test]
    fn parsing_dimensions_from_string_will_fail_for_garbage_input() {
        let dimensions_str = "Absolute_x_garbage";
        match Dimensions::from_str(dimensions_str) {
            Err(msg) => {
                assert_eq!(msg, "Failed to parse width (expected: unsigned 16-bit integer, received: \"Absolute_\")")
            }
            _ => {
                panic!("Dimensions::from_str() should have failed for input \"{}\"", dimensions_str)
            }
        }
    }

    #[test]
    fn parsing_dimensions_from_string_will_fail_for_garbage_height_even_when_width_is_valid() {
        let dimensions_str = "10xthree";
        match Dimensions::from_str(dimensions_str) {
            Err(msg) => {
                assert_eq!(msg, "Failed to parse height (expected: unsigned 16-bit integer, received: \"three\")")
            }
            _ => {
                panic!("Dimensions::from_str() should have failed for input \"{}\"", dimensions_str)
            }
        }
    }

    #[test]
    fn empty_dimensions_string_results_in_empty_dimensions_object() {
        let dimensions_str = "";
        let dimensions = Dimensions::from_str(dimensions_str).unwrap();
        assert_eq!(false, dimensions.is_width_defined());
        assert_eq!(false, dimensions.is_height_defined());
    }

    #[test]
    fn calculated_dimensions_string_results_in_empty_dimensions_object() {
        let dimensions_str = "calculated";
        let dimensions = Dimensions::from_str(dimensions_str).unwrap();
        assert_eq!(false, dimensions.is_width_defined());
        assert_eq!(false, dimensions.is_height_defined());
    }

    #[test]
    fn dimensions_can_be_printed_as_strings() {
        let d = Dimensions { width: 16, height: 9 };
        assert_eq!("16x9", d.to_string());
    }
}
