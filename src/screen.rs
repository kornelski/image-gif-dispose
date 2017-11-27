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
//! ```

use gif;
use disposal::Disposal;
use rgb::*;
use imgref::*;
use subimage::Subimage;
use std::io;
use super::Error;

/// Combined GIF frames forming a "virtual screen"
///
/// Pixel type can be `RGB8` or `RGBA8`. The size is overall GIF size (grater or equal individual frame sizes).
pub struct Screen<PixelType: Copy = RGBA8> {
    /// Result of combining all frames so far. It's in RGB/RGBA.
    pub pixels: ImgVec<PixelType>,

    global_pal: Option<Vec<PixelType>>,
    bg_color: PixelType,
    disposal: Disposal<PixelType>,
}

impl Screen<RGBA8> {
    /// Initialize an empty RGBA screen from the GIF Reader.
    ///
    /// Make sure Reader is set to use Indexed color.
    /// `decoder.set(gif::ColorOutput::Indexed);`
    pub fn new_reader<T: io::Read>(reader: &gif::Reader<T>) -> Self {
        Self::from_reader(reader)
    }
}

impl<PixelType: From<RGB8> + Copy + Default> Screen<PixelType> {
    /// Create an new `Screen` with any pixel type
    ///
    /// You may need type hints or use the `screen.pixels` to tell Rust whether you want `RGB8` or `RGBA8`.
    pub fn from_reader<T: io::Read>(reader: &gif::Reader<T>) -> Self {
        let pal = reader.global_palette().map(convert_pixels);

        let bg_color = if let (Some(bg_index), Some(pal)) = (reader.bg_color(), pal.as_ref()) {
            pal[bg_index]
        } else {
            PixelType::default()
        };
        Self::new(reader.width() as usize, reader.height() as usize, bg_color, pal)
    }

    pub fn new(width: usize, height: usize, bg_color: PixelType, global_pal: Option<Vec<PixelType>>) -> Self {
        Screen {
            pixels: Img::new(vec![bg_color; width * height], width, height),
            global_pal,
            bg_color,
            disposal: Disposal::default(),
        }
    }


    /// Advance the screen by one frame.
    ///
    /// The result will be in `screen.pixels.buf`
    pub fn blit_frame(&mut self, frame: &gif::Frame) -> Result<ImgRef<PixelType>, Error> {
        let local_pal : Option<Vec<_>> = frame.palette.as_ref().map(|bytes| convert_pixels(bytes));
        self.blit(local_pal.as_ref().map(|x| &x[..]), frame.dispose,
            frame.left, frame.top,
            ImgRef::new(&frame.buffer, frame.width as usize, frame.height as usize), frame.transparent)
    }

    pub fn blit(&mut self, local_pal: Option<&[PixelType]>, method: gif::DisposalMethod, left: u16, top: u16, buffer: ImgRef<u8>, transparent: Option<u8>) -> Result<ImgRef<PixelType>, Error> {
        let pal = local_pal.or(self.global_pal.as_ref().map(|x| &x[..])).ok_or(Error::NoPalette)?;

        let stride = self.pixels.width();
        self.disposal.dispose(&mut self.pixels.buf, stride, self.bg_color);
        self.disposal = Disposal::new(method, left, top, buffer.width() as u16, buffer.height() as u16, self.pixels.as_ref());

        for (dst, &src) in self.pixels.buf.iter_mut().subimage(left as usize, top as usize, buffer.width(), buffer.height(), stride).zip(buffer.iter()) {
            if Some(src) == transparent {
                continue;
            }
            *dst = pal[src as usize];
        }

        Ok(self.pixels.as_ref())
    }
}

fn convert_pixels<T: From<RGB8>>(palette_bytes: &[u8]) -> Vec<T> {
    palette_bytes.chunks(3).map(|byte| RGB8{r:byte[0], g:byte[1], b:byte[2]}.into()).collect()
}

#[test]
fn screen_rgb_rgba() {
    let _ = Screen::new(1,1, RGBA8::new(0,0,0,0), None);
    let _ = Screen::new(1,1, RGB8::new(0,0,0), None);
}
