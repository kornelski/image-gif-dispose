use super::Error;
use crate::disposal::Disposal;
use imgref::*;
use rgb::*;
use std::io;

/// Combined GIF frames forming a "virtual screen"
///
/// Pixel type can be `RGB8` or `RGBA8`. The size is overall GIF size (grater or equal individual frame sizes).
pub struct Screen<PixelType = RGBA8> {
    /// Result of combining all frames so far. It's in RGB/RGBA.
    pub pixels: ImgVec<PixelType>,

    global_pal: Option<Vec<PixelType>>,
    disposal: Disposal<PixelType>,
}

impl Screen<RGBA8> {
    /// Initialize an empty RGBA screen from the GIF Reader.
    ///
    /// Make sure Reader is set to use Indexed color.
    /// `options.set_color_output(gif::ColorOutput::Indexed);`
    pub fn new_decoder<T: io::Read>(reader: &gif::Decoder<T>) -> Self {
        Self::from_decoder(reader)
    }
}

impl<PixelType: From<RGB8> + Copy + Default> Screen<PixelType> {
    /// Create an new `Screen` with any pixel type
    ///
    /// You may need type hints or use the `screen.pixels` to tell Rust whether you want `RGB8` or `RGBA8`.
    pub fn from_decoder<T: io::Read>(reader: &gif::Decoder<T>) -> Self {
        let pal = reader.global_palette().map(convert_pixels);

        Self::new(reader.width().into(), reader.height().into(), PixelType::default(), pal)
    }

    /// Manual setup of the canvas. You probably should use `from_reader` instead.
    ///
    /// `bg_color` argument will be ignored. It appears that nobody tries to follow the GIF spec,
    /// and background must always be transparent.
    #[inline]
    pub fn new(width: usize, height: usize, _bg_color: PixelType, global_pal: Option<Vec<PixelType>>) -> Self {
        Screen {
            pixels: Img::new(vec![PixelType::default(); width * height], width, height),
            global_pal,
            disposal: Disposal::default(),
        }
    }

    /// Advance the screen by one frame.
    ///
    /// The result will be in `screen.pixels.buf`
    pub fn blit_frame(&mut self, frame: &gif::Frame<'_>) -> Result<ImgRef<'_, PixelType>, Error> {
        let local_pal : Option<Vec<_>> = frame.palette.as_ref().map(|bytes| convert_pixels(bytes));
        self.blit(local_pal.as_deref(), frame.dispose,
            frame.left, frame.top,
            ImgRef::new(&frame.buffer, frame.width.into(), frame.height.into()), frame.transparent)
    }

    /// Low-level version of `blit_frame`
    pub fn blit(&mut self, local_pal: Option<&[PixelType]>, method: gif::DisposalMethod, left: u16, top: u16, buffer: ImgRef<'_, u8>, transparent: Option<u8>) -> Result<ImgRef<'_, PixelType>, Error> {
        let mut pal = local_pal.or(self.global_pal.as_deref()).ok_or(Error::NoPalette)?;
        let mut tmp;
        // For backwards-compat only
        if pal.len() < 256 {
            tmp = Vec::with_capacity(256);
            tmp.extend_from_slice(pal);
            tmp.resize(256, Default::default());
            pal = &tmp;
        };
        // Some images contain out-of-pal colors. The fastest way is to extend the palette instead of doing per-pixel checks.
        let pal = &pal[0..256];

        self.disposal.dispose(self.pixels.as_mut());
        self.disposal = Disposal::new(method, left, top, buffer.width() as u16, buffer.height() as u16, self.pixels.as_ref());

        for (dst, src) in self.pixels.sub_image_mut(left.into(), top.into(), buffer.width(), buffer.height()).pixels_mut().zip(buffer.pixels()) {
            if Some(src) == transparent {
                continue;
            }
            *dst = pal[src as usize];
        }

        Ok(self.pixels.as_ref())
    }
}

fn convert_pixels<T: From<RGB8> + Default>(palette_bytes: &[u8]) -> Vec<T> {
    let mut res = Vec::with_capacity(256);
    res.extend(palette_bytes.chunks(3).map(|byte| RGB8{r:byte[0], g:byte[1], b:byte[2]}.into()));
    while res.len() < 256 {
        res.push(Default::default());
    }
    res
}

#[test]
fn screen_rgb_rgba() {
    let _ = Screen::new(1, 1, RGBA8::new(0, 0, 0, 0), None);
    let _ = Screen::new(1, 1, RGB8::new(0, 0, 0), None);
}
