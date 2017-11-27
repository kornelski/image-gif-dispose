//! Implements GIF disposal method for the gif crate.
//!
//! The gif crate only exposes raw frame data that is not sufficient
//! to render GIFs properly. GIF requires special composing of frames
//! which, as this crate shows, is non-trivial.
//!
//! ```rust,ignore
//! let file = File::open("example.gif")?;
//! let mut decoder = Decoder::new(file);
//!
//! // Important:
//! decoder.set(gif::ColorOutput::Indexed);
//!
//! let mut reader = decoder.read_info()?;
//!
//! let mut screen = Screen::new_reader(&reader);
//! while let Some(frame) = reader.read_next_frame()? {
//!     screen.blit(&frame)?;
//!     screen.pixels // that's the frame now
//! }
//!

extern crate gif;
extern crate rgb;
extern crate imgref;

mod subimage;
mod disposal;
mod screen;

pub use screen::Screen;
pub use rgb::{RGB8, RGBA8};

use std::fmt;
use std::error::Error as StdError;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Error {
    NoPalette
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.description())
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        return "No palette";
    }
}
