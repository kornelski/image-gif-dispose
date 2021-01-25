use super::Error;
use crate::disposal::Disposal;
use imgref::*;
use rgb::*;
use std::io;

/// Combined GIF frames forming a "virtual screen". See [Screen::new_decoder].
///
/// Pixel type can be `RGB8` or `RGBA8`. The size is overall GIF size (grater or equal individual frame sizes).
pub struct Screen<PixelType = RGBA8> {
    /// Result of combining all frames so far. It's in RGB/RGBA.
    pub pixels: ImgVec<PixelType>,

    global_pal: Option<Vec<PixelType>>,
    disposal: Disposal<PixelType>,
}

impl Screen<RGBA8> {
    /// Create an new `Screen` with RGBA pixel type (the best choice for GIF)
    ///
    /// Make sure Reader is set to use `Indexed` color.
    /// `options.set_color_output(gif::ColorOutput::Indexed);`
    pub fn new_decoder<T: io::Read>(reader: &gif::Decoder<T>) -> Self {
        Self::from_decoder(reader)
    }
}

impl<PixelType: From<RGB8> + Copy + Default> Screen<PixelType> {
    /// Create an new `Screen` with either `RGB8` or `RGBA8` pixel type. Allows ignoring transparency.
    ///
    /// You may need type hints or use the `screen.pixels` to tell Rust whether you want `RGB8` or `RGBA8`.
    #[must_use]
    pub fn from_decoder<T: io::Read>(reader: &gif::Decoder<T>) -> Self {
        let w = reader.width();
        let h = reader.height();
        let pal = reader.global_palette().map(convert_pixels);

        Self::new(w.into(), h.into(), PixelType::default(), pal)
    }

    /// Manual setup of the canvas. You probably should use `from_reader` instead.
    ///
    /// `bg_color` argument will be ignored. It appears that nobody tries to follow the GIF spec,
    /// and background must always be transparent.
    #[inline]
    #[must_use]
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
        self.dispose().then_blit(local_pal, method, left, top, buffer, transparent)?;
        Ok(self.pixels.as_ref())
    }

    fn blit_without_dispose(&mut self, local_pal: Option<&[PixelType]>, method: gif::DisposalMethod, left: u16, top: u16, buffer: ImgRef<'_, u8>, transparent: Option<u8>) -> Result<(), Error> {
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

        self.disposal = Disposal::new(method, left, top, buffer.width() as u16, buffer.height() as u16, self.pixels.as_ref());

        for (dst, src) in self.pixels.sub_image_mut(left.into(), top.into(), buffer.width(), buffer.height()).pixels_mut().zip(buffer.pixels()) {
            if Some(src) == transparent {
                continue;
            }
            *dst = pal[src as usize];
        }
        Ok(())
    }

    #[inline(always)]
    #[doc(hidden)]
    pub fn pixels(&mut self) -> ImgRef<'_, PixelType> {
        self.pixels.as_ref()
    }

    /// Advanced usage. You do not need to call this. It exposes an incompletely-drawn screen.
    ///
    /// Call to this method must always be followed by `.then_blit()` to fix the incomplete state.
    ///
    /// The state is after previous frame has been disposed, but before the next frame has been drawn.
    /// This state is never visible on screen.
    ///
    /// This method is for GIF encoders to help find minimal difference between frames, especially
    /// when transparency is involved ("background" disposal method).
    ///
    /// ```rust
    /// # fn example(buffer: imgref::ImgRef<u8>) -> Result<(), gif_dispose::Error> {
    /// use gif_dispose::*;
    /// let mut screen = Screen::new(320, 200, RGBA8::new(0,0,0,0), None);
    /// let mut tmp_screen = screen.dispose();
    /// let incomplete_pixels = tmp_screen.pixels();
    /// tmp_screen.then_blit(None, gif::DisposalMethod::Keep, 0, 0, buffer, None)?;
    /// # Ok(()) }
    /// ```
    #[inline]
    pub fn dispose(&mut self) -> TempDisposedStateScreen<'_, PixelType> {
        self.disposal.dispose(self.pixels.as_mut());
        TempDisposedStateScreen(self)
    }
}


/// Screen that has a temporary state between frames
#[must_use]
pub struct TempDisposedStateScreen<'screen, PixelType>(&'screen mut Screen<PixelType>);

/// Extends borrow to the end of scope, reminding to use `then_blit`
impl<T> Drop for TempDisposedStateScreen<'_, T> {
    fn drop(&mut self) {
    }
}

impl<'s, PixelType: From<RGB8> + Copy + Default> TempDisposedStateScreen<'s, PixelType> {
    #[inline(always)]
    pub fn then_blit(self, local_pal: Option<&[PixelType]>, method: gif::DisposalMethod, left: u16, top: u16, buffer: ImgRef<'_, u8>, transparent: Option<u8>) -> Result<(), Error> {
        self.0.blit_without_dispose(local_pal, method, left, top, buffer, transparent)
    }

    /// Access pixels in the in-between state
    #[inline(always)]
    pub fn pixels(&mut self) -> ImgRef<'_, PixelType> {
        self.0.pixels.as_ref()
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
