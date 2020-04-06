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
//!     screen.blit_frame(&frame)?;
//!     screen.pixels // that's the frame now
//! }
//!

mod disposal;
mod screen;

pub use crate::screen::Screen;
pub use rgb::{RGB8, RGBA8};

use std::error::Error as StdError;
use std::fmt;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Error {
    NoPalette,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("No palette")
    }
}

impl StdError for Error {}
