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
use std::error::Error;

pub struct Screen {
    /// Result of combining frames
    pub pixels: ImgVec<RGBA8>,

    global_pal: Option<Vec<RGBA8>>,
    bg_color: RGBA8,
    disposal: Disposal<RGBA8>,
}

impl Screen {

    /// Initialize empty screen from GIF Reader.
    ///
    /// Make sure Reader is set to use Indexed color.
    /// `decoder.set(gif::ColorOutput::Indexed);`
    pub fn new_reader<T: io::Read>(reader: &gif::Reader<T>) -> Self {
        let pal = reader.global_palette().map(|palette_bytes| to_rgba(palette_bytes));

        let bg_color = if let (Some(bg_index), Some(pal)) = (reader.bg_color(), pal.as_ref()) {
            pal[bg_index]
        } else {
            RGBA8::new(0,0,0,0)
        };
        Self::new(reader.width() as usize, reader.height() as usize, bg_color, pal)
    }

    pub fn new(width: usize, height: usize, bg_color: RGBA8, global_pal: Option<Vec<RGBA8>>) -> Self {
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
    pub fn blit_frame(&mut self, frame: &gif::Frame) -> Result<ImgRef<RGBA8>, Box<Error>> {
        let local_pal : Option<Vec<_>> = frame.palette.as_ref().map(|bytes| to_rgba(bytes));
        self.blit(local_pal.as_ref().map(|x| &x[..]), frame.dispose,
            frame.left, frame.top,
            ImgRef::new(&frame.buffer, frame.width as usize, frame.height as usize), frame.transparent)
    }

    pub fn blit(&mut self, local_pal: Option<&[RGBA8]>, method: gif::DisposalMethod, left: u16, top: u16, buffer: ImgRef<u8>, transparent: Option<u8>) -> Result<ImgRef<RGBA8>, Box<Error>> {
        let pal = local_pal.or(self.global_pal.as_ref().map(|x| &x[..])).ok_or("the frame must have _some_ palette")?;

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

fn to_rgba(palette_bytes: &[u8]) -> Vec<RGBA8> {
    palette_bytes.chunks(3).map(|byte| RGBA8{r:byte[0], g:byte[1], b:byte[2], a:255}).collect()
}
