use gif::DisposalMethod;
use imgref::*;
use std::default::Default;

enum SavedState<Pixel> {
    Previous(Vec<Pixel>),
    Background,
    Keep,
}

pub struct Disposal<Pixel> {
    saved: SavedState<Pixel>,
    left: u16, top: u16,
    width: u16, height: u16,
}

impl<Pixel: Copy + Default> Default for Disposal<Pixel> {
    fn default() -> Self {
        Disposal {
           saved: SavedState::Keep,
           top: 0, left: 0, width: 0, height: 0,
       }
   }
}

impl<Pixel: Copy + Default> Disposal<Pixel> {
    pub fn dispose(&self, mut pixels: ImgRefMut<'_, Pixel>) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        let mut dest = pixels.sub_image_mut(self.left.into(), self.top.into(), self.width.into(), self.height.into());
        match &self.saved {
            SavedState::Background => {
                let bg = Pixel::default();
                for px in dest.pixels_mut() { *px = bg; }
            },
            SavedState::Previous(saved) => {
                for (px, &src) in dest.pixels_mut().zip(saved.iter()) { *px = src; }
            },
            SavedState::Keep => {},
        }
    }

    pub fn new(method: gif::DisposalMethod, left: u16, top: u16, width: u16, height: u16, pixels: ImgRef<'_, Pixel>) -> Self {
        Disposal {
            saved: match method {
                DisposalMethod::Previous => SavedState::Previous(pixels.sub_image(left.into(), top.into(), width.into(), height.into()).pixels().collect()),
                DisposalMethod::Background => SavedState::Background,
                _ => SavedState::Keep,
            },
            left, top, width, height,
        }
    }
}
