//! Implements GIF disposal method for the [gif crate](https://lib.rs/crates/gif).
//!
//! The gif crate only exposes raw frame data that is not sufficient
//! to render GIFs properly. GIF requires special composing of frames
//! which, as this crate shows, is non-trivial.
//!
//! ```rust,no_run
//! let file = std::fs::File::open("example.gif")?;
//!
//! let mut gif_opts = gif::DecodeOptions::new();
//! // Important:
//! gif_opts.set_color_output(gif::ColorOutput::Indexed);
//!
//! let mut decoder = gif_opts.read_info(file)?;
//! let mut screen = gif_dispose::Screen::new_decoder(&decoder);
//!
//! while let Some(frame) = decoder.read_next_frame()? {
//!     screen.blit_frame(&frame)?;
//!     let _ = screen.pixels_rgba().clone(); // that's the frame now in RGBA format
//!     let _ = screen.pixels_rgba().to_contiguous_buf(); // that works too
//! }
//! # Ok::<_, Box<dyn std::error::Error>>(())
//! ```

mod disposal;
mod screen;

pub use crate::screen::Screen;
pub use crate::screen::TempDisposedStateScreen;
pub use rgb::{RGB8, RGBA8};
pub use imgref::ImgRef;

use std::error::Error as StdError;
use std::fmt;

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
#[non_exhaustive]
pub enum Error {
    /// GIF must have either a global palette set, or per-frame palette set. If there is none, it's not possible to render.
    NoPalette,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("No palette")
    }
}

impl StdError for Error {}
