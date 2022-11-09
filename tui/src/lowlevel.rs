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

use std::fmt::Display;
use std::io::Write;

use crossterm;
use xy_utils::{Dimensions, Point};

/// Enumeration of colors that can be applied to the plotters paintbrush.
///
/// This enumeration provides several different color definition options:
///
/// * Named ANSI colors (e.g. Black, Blue, Magenta, etc).
/// * The "Unset" special color, which resets colors to the underlying default.
/// * The "Rgb" color, which accepts red, green, and blue values for custom
///   colors.
///
/// Note the existence of the "Unset" color. This exists because the way that
/// terminals make color changes is persistent. So if you change the foreground
/// color of a terminal (to re-color) the terminal text, it will remain that
/// color until changed again.
///
/// The "Unset" command is an explicit request to the terminal to "restore
/// all colors to their default values, as though we had not manually changed
/// them".
///
/// ## Examples
///
/// ### Example 1: Defining a Basic Color
///
/// ```
/// use tui::Color;
///
/// let color = Color::Cyan;
/// assert_eq!(Color::Cyan, color);
/// ```
///
/// ### Example 2: Defining a Custom Color
///
/// ```
/// use tui::Color;
///
/// // Making pure cyan, regardless of terminal settings.
/// let color = Color::Rgb(0, 255, 255);
/// assert_eq!(Color::Rgb(0, 255, 255), color);
/// ```
///
/// ### Example 3: Resetting Colors.
///
/// ```
/// use tui::Color;
///
/// // We don't want fancy colors any more.
/// let color = Color::Unset;
/// assert_eq!(Color::Unset, color);
/// ```
///
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Color {
    /// Unset color resets the color to the terminal default.
    Unset,
    Rgb(u8, u8, u8),
    Black,
    Blue,
    Cyan,
    DarkBlue,
    DarkCyan,
    DarkGreen,
    DarkGrey,
    DarkMagenta,
    DarkRed,
    DarkYellow,
    Green,
    Grey,
    Magenta,
    Red,
    White,
    Yellow,
}

/// Color settings for the plotter. Any content written to the TUI will use the
/// color settings applied to the paintbrush. The paintbrush can set three
/// separate values:
///
/// * Foreground color (`fg`): The color of the text characters
/// * Background color (`bg`): The color _behind_ the text characters.
/// * Stroke hardness (`bold`): When true, tells the terminal to enhance the
///   text.
///
/// ## Example
///
/// ```
/// use tui::{Paintbrush, Color};
///
/// // This is the ugliest color combination I can imagine.
/// let pb = Paintbrush {
///     fg: Color::Cyan,
///     bg: Color::Magenta,
///     bold: true
/// };
/// ```
#[derive(Clone, Debug)]
pub struct Paintbrush {
    /// Foreground color.
    pub fg: Color,

    /// Background color.
    pub bg: Color,

    /// Stroke hardness (set to true to tell terminal to enhance the text for
    /// the users attention).
    pub bold: bool,
}

impl Paintbrush {
    /// Create the "default" paintbrush.
    ///
    /// This function returns a paintbrush that has no specific color settings,
    /// and simply uses the terminal emulator's default colors.
    pub fn create_default() -> Paintbrush {
        Paintbrush { fg: Color::Unset, bg: Color::Unset, bold: false }
    }
}

/// The Plotter trait defines the implementation skeleton for an object that can
/// be used to perform content rendering.
///
/// Plotter commands are queued before they changes are visible on-screen. For
/// efficiency, once all changes have been applied to the plotter, call
/// `flush()` to commit the changes and show them on screen.
///
/// Plotters are expected to be "lazy evaluating" This means that plotters
/// should try to queue up changes into batches, and then only flush those
/// changes when:
///
/// 1. The user is ready, and calls the `flush()` function, or
/// 2. The underlying command buffer is full, and a flush of all commands is
///    forced.
///
/// ## Example
///
/// ```
/// # use tui::mock::MockPlotter as MyPlotter;
/// use tui::{Plotter, Paintbrush, Color};
/// use xy_utils::Point;
///
/// // This code assumes that `MyPlotter` is a user-defined plotter. A default
/// // TUI plotter is provided (called `DefaultPlotter`) that should be suitable
/// // for the majority of use cases.
/// let mut plotter = MyPlotter::new();
///
/// plotter.set_paintbrush(&Paintbrush::create_default()).unwrap();
/// plotter.plot(Point{ x: 0, y: 0 }, "FOO").unwrap();
/// plotter.flush().unwrap();
/// ```
///
/// ## Better Abstractions Are Available!
///
/// While it is perfectly allowed to call the `set_paintbrush` and `plot`
/// functions directly, you should also take a look at the higher level
/// `components` classes (e.g. `tui::components::Canvas`) which perform a lot
/// of automation of this work. You will still need to manually flush the
/// plotter when you decide the UI should be updated, but you can take
/// advantage of the `tui::components` module to make your life easier.
///
/// ```
/// # use tui::mock::MockPlotter as MyPlotter;
/// use tui::{Plotter, Paintbrush, Color};
/// use tui::components::TextLabel;
/// use xy_utils::{Dimensions, Point};
///
/// let mut plotter = MyPlotter::new();
/// let label = TextLabel::new(
///     Paintbrush::create_default(),
///     Point { x: 1, y: 2 },
///     Dimensions {
///         width: 3,
///         height: 1,
///     },
///     "FOO",
/// );
///
/// // Use our plotter to render the label.
/// label.render(&mut plotter).unwrap();
/// plotter.flush().unwrap();
/// ```
pub trait Plotter {
    /// Get the plot area.
    ///
    /// The dimensions returned are the total area in which terminal characters
    /// can be rendered.
    fn get_plot_area(&self) -> Dimensions;

    /// Set the paintbrush that defines the output style for future plotted
    /// objects.
    ///
    /// The paintbrush defines the foreground color, background color, and
    /// emphasis traits of plotted characters.
    ///
    /// ## Arguments
    ///
    /// * `pb`: The paintbrush to use for the next series of plotted objects.
    ///
    /// ## Returns
    ///
    /// A result object. In the success case, a reference to `self` is returned
    /// to allow for a fluent interface, otherwise, an `std::io::Error` is
    /// returned that will indicate any issues encountered.
    ///
    /// ## Example
    ///
    /// ```
    /// # use tui::mock::MockPlotter as MyPlotter;
    /// use tui::{Plotter, Paintbrush, Color};
    /// use tui::components::TextLabel;
    /// use xy_utils::{Dimensions, Point};
    ///
    /// let mut plotter = MyPlotter::new();
    /// let paintbrush = Paintbrush::create_default();
    ///
    /// // Make a red piece of text.
    /// plotter
    ///     .set_paintbrush(&Paintbrush {fg: Color::Red, ..paintbrush.clone()}).unwrap()
    ///     .plot(Point{x: 0, y: 0}, "Foo").unwrap();
    ///
    /// // Make a blue piece of text next.
    /// plotter
    ///     .set_paintbrush(&Paintbrush {fg: Color::Blue, ..paintbrush.clone()}).unwrap()
    ///     .plot(Point{x: 0, y: 1}, "Bar").unwrap();
    ///
    /// // Finally make an "uncolored" piece of text that uses the terminal's
    /// // default colors.
    /// plotter
    ///     .set_paintbrush(&Paintbrush::create_default()).unwrap()
    ///     .plot(Point{x: 0, y: 2}, "Baz").unwrap();
    ///
    /// plotter.flush().unwrap();
    /// ```
    fn set_paintbrush(&mut self, pb: &Paintbrush) -> Result<&mut Self, std::io::Error>;

    /// Plot content to the UI. This can be any value that implements the
    /// Display trait.
    ///
    /// ## Arguments
    ///
    /// * `location`: Where to start printing the content. Content is printed
    ///               along the X-axis (i.e. left-to-right).
    /// * `content`: Content to plot This can be any value implementing the
    ///              std::fmt::Display trait.
    fn plot<T: Display>(
        &mut self,
        location: Point,
        content: T,
    ) -> Result<&mut Self, std::io::Error>;

    /// Flush any queued changes to the user interface.
    fn flush(&mut self) -> Result<&mut Self, std::io::Error>;
}

/// Default plotter implementation.
///
/// Uses the [crossterm][1] crate to do TUI rendering.
///
/// ## `OutputStream` Type
///
/// This generic type can be any type that implements the `std::io::Write`
/// trait. Out of the box, the DefaultPlotter class supports std::io::stdout
/// which the user can create with the `DefaultPlotter::create_from_stdout()`
/// function.
///
/// ## Examples
///
/// ### Example 1: Standard usage:
///
/// ```
/// use tui::DefaultPlotter;
/// use tui::Plotter;
/// use xy_utils::{Dimensions, Point};
///
/// let mut plotter = DefaultPlotter::create_from_stdout();
/// plotter.plot(Point{x: 0, y: 10 }, "FOO").unwrap();
/// ```
///
/// [1]: https://github.com/crossterm-rs/crossterm.
pub struct DefaultPlotter<OutputStream: Write> {
    /// Output stream (e.g. stdout)
    outstream: OutputStream,
}

impl<OutputStream: Write> Plotter for DefaultPlotter<OutputStream> {
    #[cfg(not(tarpaulin_include))]
    fn get_plot_area(&self) -> Dimensions {
        let (width, height) = crossterm::terminal::size().unwrap();
        Dimensions { width: width as usize, height: height as usize }
    }

    #[cfg(not(tarpaulin_include))]
    fn set_paintbrush(&mut self, pb: &Paintbrush) -> Result<&mut Self, std::io::Error> {
        use crossterm::style::*;
        crossterm::queue!(
            self.outstream,
            SetForegroundColor(Self::convert_color_to_crossterm_val(&pb.fg)),
            SetBackgroundColor(Self::convert_color_to_crossterm_val(&pb.bg)),
            //SetAttribute(if pb.bold { Attribute::Bold } else { Attribute::NoBold })
        )?;

        Ok(self)
    }

    #[cfg(not(tarpaulin_include))]
    fn plot<T: Display>(
        &mut self,
        location: Point,
        content: T,
    ) -> Result<&mut Self, std::io::Error> {
        crossterm::queue!(
            self.outstream,
            crossterm::cursor::MoveTo(location.x as u16, location.y as u16),
            crossterm::style::Print(content)
        )?;
        Ok(self)
    }

    #[cfg(not(tarpaulin_include))]
    fn flush(&mut self) -> Result<&mut Self, std::io::Error> {
        self.outstream.flush()?;
        Ok(self)
    }
}

impl<OutputStream: Write> Drop for DefaultPlotter<OutputStream> {
    #[cfg(not(tarpaulin_include))]
    fn drop(&mut self) {
        // Must leave alternate screen mode.
        crossterm::execute!(self.outstream, crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
    }
}

impl<OutputStream: Write> DefaultPlotter<OutputStream> {
    pub fn new(outstream: OutputStream) -> DefaultPlotter<OutputStream> {
        let mut result = DefaultPlotter { outstream };
        result.reset();
        result
    }

    /// Internal function to reset the terminal before initializing the UI.
    fn reset(&mut self) {
        // This terminal command does three things:
        //
        // * Enters "Alternate Screen Mode" (see: https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#The%20Alternate%20Screen%20Buffer).
        // * Resets any custom colors applied by the parent process.
        // * Removes any custom attribues applied by the parent process.
        //
        // The destructor will leave alternate screen mode when the plotter is
        // destroyed.
        crossterm::execute!(
            self.outstream,
            crossterm::terminal::EnterAlternateScreen,
            crossterm::style::ResetColor,
            crossterm::style::SetAttribute(crossterm::style::Attribute::Reset),
            crossterm::style::SetAttribute(crossterm::style::Attribute::NoUnderline)
        )
        .unwrap();

        crossterm::terminal::enable_raw_mode().unwrap();

        // Clear the entire plotting area.
        self.set_paintbrush(&Paintbrush::create_default()).unwrap();
        let mut blanker = String::with_capacity(self.get_plot_area().width);
        for _ in 0..self.get_plot_area().width {
            blanker.push_str(" ");
        }
        for i in 0..self.get_plot_area().height {
            self.plot(Point { x: 0, y: i }, &blanker).unwrap();
        }
    }

    /// Helper function to convert a local crate color to a crossterm version.
    #[cfg(not(tarpaulin_include))]
    fn convert_color_to_crossterm_val(internal_color: &Color) -> crossterm::style::Color {
        match internal_color {
            Color::Unset => crossterm::style::Color::Reset,
            Color::Rgb(r, g, b) => crossterm::style::Color::Rgb { r: *r, g: *g, b: *b },
            Color::Black => crossterm::style::Color::Black,
            Color::Blue => crossterm::style::Color::Blue,
            Color::Cyan => crossterm::style::Color::Cyan,
            Color::DarkBlue => crossterm::style::Color::DarkBlue,
            Color::DarkCyan => crossterm::style::Color::DarkCyan,
            Color::DarkGreen => crossterm::style::Color::DarkGreen,
            Color::DarkGrey => crossterm::style::Color::DarkGrey,
            Color::DarkMagenta => crossterm::style::Color::DarkMagenta,
            Color::DarkRed => crossterm::style::Color::DarkRed,
            Color::DarkYellow => crossterm::style::Color::DarkYellow,
            Color::Green => crossterm::style::Color::Green,
            Color::Grey => crossterm::style::Color::Grey,
            Color::Magenta => crossterm::style::Color::Magenta,
            Color::Red => crossterm::style::Color::Red,
            Color::White => crossterm::style::Color::White,
            Color::Yellow => crossterm::style::Color::Yellow,
        }
    }
}

impl DefaultPlotter<std::io::Stdout> {
    pub fn create_from_stdout() -> DefaultPlotter<std::io::Stdout> {
        DefaultPlotter::<std::io::Stdout>::new(std::io::stdout())
    }
}

/// The mock module provides some basic mock implementations of TUI objects that
/// can be introspected as part of unit tests.
#[cfg(not(tarpaulin_include))]
pub mod mock {
    use super::*;

    /// Enumeration that holds the possible commands executed by the mock
    /// plotter.
    #[derive(Debug)]
    pub enum MockPlotterCommand {
        /// User set the paintbrush.
        SetPaintbrush(Paintbrush),

        /// User plotted an object. Actual object is stored as a string.
        PlotObject(Point, String),

        /// UI changes were flushed.
        Flush,
    }

    /// Mock Plotter implementation for testing.
    pub struct MockPlotter {
        pub plot_area: Dimensions,
        pub command_list: Vec<MockPlotterCommand>,
    }

    impl Plotter for MockPlotter {
        fn get_plot_area(&self) -> Dimensions {
            self.plot_area
        }

        fn set_paintbrush(&mut self, pb: &Paintbrush) -> Result<&mut Self, std::io::Error> {
            self.command_list.push(MockPlotterCommand::SetPaintbrush(pb.clone()));
            Ok(self)
        }

        fn plot<T: Display>(
            &mut self,
            location: Point,
            content: T,
        ) -> Result<&mut Self, std::io::Error> {
            self.command_list
                .push(MockPlotterCommand::PlotObject(location, format!("{}", content)));
            Ok(self)
        }

        fn flush(&mut self) -> Result<&mut Self, std::io::Error> {
            self.command_list.push(MockPlotterCommand::Flush);
            Ok(self)
        }
    }

    impl MockPlotter {
        pub fn new() -> MockPlotter {
            MockPlotter { plot_area: Dimensions { height: 20, width: 20 }, command_list: vec![] }
        }
    }
}

#[cfg(test)]
mod paintbrush_tests {
    use super::*;

    #[test]
    fn create_default_function_creates_a_paintbrush_that_will_use_default_terminal_colors() {
        let pb = Paintbrush::create_default();

        assert_eq!(Color::Unset, pb.fg);
        assert_eq!(Color::Unset, pb.bg);
        assert_eq!(false, pb.bold);
    }
}

#[cfg(test)]
mod default_plotter_tests {
    use crate::DefaultPlotter;

    #[test]
    fn create_from_stdout_function_returns_a_default_plotter() {
        let _ = DefaultPlotter::create_from_stdout();
    }
}
