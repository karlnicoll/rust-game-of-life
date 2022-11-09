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

//! The `tui` library is an extremely simple terminal user interface component
//! crate that provides wrappers around manual terminal UI rendering.
//!
//! It has two core concepts. Plotters and Components.
//!
//! ## Plotters
//!
//! Plotters are the thing that draws onto the terminal. Plotters are very basic
//! objects that, given a "paintbrush", a displayable object, and a set of
//! coordinates. Will draw it onto the screen.
//!
//! ```
//! use tui::{DefaultPlotter, Paintbrush, Plotter};
//! use xy_utils::Point;
//!
//! // Plotters must be mutable to be used.
//! let mut plotter = DefaultPlotter::create_from_stdout();
//!
//! // Set the "paintbrush". A default can be used, or you can provide your own
//! // paintbrush for settings colors.
//! plotter.set_paintbrush(&Paintbrush::create_default()).unwrap();
//!
//! // Plot a string to the top left corner of the terminal.
//! plotter.plot(Point{ x: 0, y: 0 }, "FOO").unwrap();
//!
//! // Flush the plotter to commit your changes and draw to the screen.
//! plotter.flush().unwrap();
//! ```
//!
//! ## Components
//!
//! Components are objects that provide abstractions for commonly used TUI widgets.
//!
//! There are currently four main types of widget:
//!
//! ### Canvas
//!
//! Perhaps the most basic component. Draw stuff within a boundary defined by a
//! point and size:
//!
//! ```
//! use tui::{DefaultPlotter, Paintbrush, Plotter};
//! use tui::components::Canvas;
//! use xy_utils::{Dimensions, Point};
//!
//! let mut plotter = DefaultPlotter::create_from_stdout();
//! let mut canvas = Canvas::new(Point { x: 1, y: 2 }, Dimensions { width: 3, height: 3 });
//!
//! canvas.draw_str(Paintbrush::create_default(), Point { x: 0, y: 0 }, "***").unwrap();
//! canvas.render(&mut plotter).unwrap();
//!
//! plotter.flush().unwrap();
//! ```
//!
//! ### TextLabel
//!
//! A basic text output box:
//!
//! ```
//! use tui::{DefaultPlotter, Paintbrush, Plotter};
//! use tui::components::TextLabel;
//! use xy_utils::{Dimensions, Point};
//!
//! let mut plotter = DefaultPlotter::create_from_stdout();
//! let label = TextLabel::new(
//!     Paintbrush::create_default(),
//!     Point { x: 1, y: 2 },
//!     Dimensions { width: 3, height: 1 },
//!     "FOO",
//! );
//!
//! label.render(&mut plotter).unwrap();
//! plotter.flush().unwrap();
//! ```
//!
//! A label is size-aware, so attempting to write too much data into the label will
//! result in it being truncated.
//!
//! ```
//! # use tui::{DefaultPlotter, Paintbrush, Plotter};
//! # use tui::components::TextLabel;
//! # use xy_utils::{Dimensions, Point};
//
//! let mut plotter = DefaultPlotter::create_from_stdout();
//! let label = TextLabel::new(
//!     Paintbrush::create_default(),
//!     Point { x: 1, y: 2 },
//!     Dimensions { width: 6, height: 1 },
//!     "FOOBARBAZ",
//! );
//!
//! label.render(&mut plotter).unwrap();
//! plotter.flush().unwrap();
//!
//! // "FOO..." will be shown on screen since the text string is bigger than the
//! // label area.
//! ```
//!
//! ### Count
//!
//! A derivative of the TextLabel that can display a count.
//!
//! ### Border
//!
//! An extremely simple widget that draws a border around a portion of the
//! terminal. Can be used to surround any other widget with a border.

pub mod components;
mod lowlevel;

// Re-export the publicly interesting types so that the user doesn't have to
// navigate the individual sub-modules.

pub use lowlevel::{mock, Color, DefaultPlotter, Paintbrush, Plotter};
