use super::Error;
use crate::disposal::Disposal;
use imgref::*;
use rgb::*;
use std::io;

/// Combined GIF frames forming a "virtual screen". See [Screen::new_decoder].
///
/// Pixel type can be `RGB8` or `RGBA8`. The size is overall GIF size (grater or equal individual frame sizes).
pub struct Screen {
    /// Result of combining all frames so far. It's in RGB/RGBA.
    internal_pixels: ImgVec<RGBA8>,

    global_pal: Option<[RGB8; 256]>,
    next_disposal: Disposal,
}

impl Screen {
    /// Create an new `Screen`
    ///
    /// Make sure Reader is set to use `Indexed` color.
    /// `options.set_color_output(gif::ColorOutput::Indexed);`
    #[must_use]
    pub fn new_decoder<T: io::Read>(reader: &gif::Decoder<T>) -> Self {
        let w = reader.width();
        let h = reader.height();
        let pal = reader.global_palette().map(|pal| pal.as_rgb());
        Self::new(w.into(), h.into(), pal)
    }

    /// Manual setup of the canvas. You probably should use `new_decoder` instead.
    ///
    /// Use `rgb` crate's `as_rgb()` if you have palette as `&[u8]`.
    #[inline]
    #[must_use]
    pub fn new(width: usize, height: usize, global_pal: Option<&[RGB8]>) -> Self {
        Screen {
            internal_pixels: Img::new(vec![RGBA8::default(); width * height], width, height),
            global_pal: global_pal.map(|g| std::array::from_fn(move |i| g.get(i).copied().unwrap_or_default())),
            next_disposal: Disposal::default(),
        }
    }

    /// Advance the screen by one frame.
    ///
    /// Use `pixels_rgba()` to get pixels afterwards
    pub fn blit_frame(&mut self, frame: &gif::Frame<'_>) -> Result<(), Error> {
        let local_pal = frame.palette.as_deref().map(|p| p.as_rgb());
        self.blit(local_pal.map(|p| &p[..]), frame.dispose,
            frame.left, frame.top,
            ImgRef::new(&frame.buffer, frame.width.into(), frame.height.into()), frame.transparent)
    }


    /// Low-level version of `blit_frame`
    pub fn blit(&mut self, local_pal: Option<&[RGB8]>, method: gif::DisposalMethod, left: u16, top: u16, buffer: ImgRef<'_, u8>, transparent: Option<u8>) -> Result<(), Error> {
        self.dispose_only().then_blit(local_pal, method, left, top, buffer, transparent)?;
        Ok(())
    }

    fn blit_without_dispose(&mut self, local_pal: Option<&[RGB8]>, method: gif::DisposalMethod, left: u16, top: u16, buffer: ImgRef<'_, u8>, transparent: Option<u8>) -> Result<(), Error> {
        self.next_disposal = Disposal::new(method, left, top, buffer.width() as u16, buffer.height() as u16, self.internal_pixels.as_ref());

        let pal_slice = local_pal.or(self.global_pal.as_ref().map(|p| &p[..])).ok_or(Error::NoPalette)?;
        let pal: [_; 256] = std::array::from_fn(|i| {
            pal_slice.get(i).copied().unwrap_or_default()
        });

        for (dst, src) in self.internal_pixels.sub_image_mut(left.into(), top.into(), buffer.width(), buffer.height()).pixels_mut().zip(buffer.pixels()) {
            if Some(src) == transparent {
                continue;
            }
            *dst = pal[src as usize].alpha(255);
        }
        Ok(())
    }

    /// Access the currently rendered pixels
    #[inline(always)]
    pub fn pixels_rgba(&mut self) -> ImgRef<'_, RGBA8> {
        self.internal_pixels.as_ref()
    }

    /// Use [`pixels_rgba`]
    #[deprecated(note = "use pixels_rgba() instead. This method will return a different type in the next version")]
    pub fn pixels(&mut self) -> ImgRef<'_, RGBA8> {
        self.pixels_rgba()
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
    /// let mut screen = Screen::new(320, 200, None);
    /// let mut tmp_screen = screen.dispose_only();
    /// let incomplete_pixels = tmp_screen.pixels();
    /// tmp_screen.then_blit(None, gif::DisposalMethod::Keep, 0, 0, buffer, None)?;
    /// # Ok(()) }
    /// ```
    #[inline]
    pub fn dispose_only(&mut self) -> TempDisposedStateScreen<'_> {
        self.next_disposal.dispose(self.internal_pixels.as_mut());
        TempDisposedStateScreen(self)
    }
}


/// Screen that has a temporary state between frames
#[must_use]
pub struct TempDisposedStateScreen<'screen>(&'screen mut Screen);

/// Extends borrow to the end of scope, reminding to use `then_blit`
impl Drop for TempDisposedStateScreen<'_> {
    fn drop(&mut self) {
    }
}

impl<'s, > TempDisposedStateScreen<'s> {
    #[inline(always)]
    pub fn then_blit(self, local_pal: Option<&[RGB8]>, method: gif::DisposalMethod, left: u16, top: u16, buffer: ImgRef<'_, u8>, transparent: Option<u8>) -> Result<(), Error> {
        self.0.blit_without_dispose(local_pal, method, left, top, buffer, transparent)
    }

    /// Access pixels in the in-between state
    #[inline(always)]
    pub fn pixels_rgba(&mut self) -> ImgRef<'_, RGBA8> {
        self.0.internal_pixels.as_ref()
    }


    /// Use [`pixels_rgba`]
    #[deprecated(note = "use pixels_rgba() instead. This method will return a different type in the next version")]
    pub fn pixels(&mut self) -> ImgRef<'_, RGBA8> {
        self.pixels_rgba()
    }
}
