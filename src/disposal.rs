use gif::DisposalMethod::*;
use imgref::*;
use std::default::Default;

pub struct Disposal<Pixel> {
    method: gif::DisposalMethod,
    previous_pixels: Option<Vec<Pixel>>,
    left: u16, top: u16,
    width: u16, height: u16,
}

impl<Pixel: Copy + Default> Default for Disposal<Pixel> {
    fn default() -> Self {
        Disposal {
           method: gif::DisposalMethod::Keep,
           previous_pixels: None,
           top: 0, left: 0, width: 0, height: 0,
       }
   }
}

impl<Pixel: Copy + Default> Disposal<Pixel> {
    pub fn dispose(&mut self, mut pixels: ImgRefMut<'_, Pixel>) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        let mut dest = pixels.sub_image_mut(self.left.into(), self.top.into(), self.width.into(), self.height.into());
        match self.method {
            Background => {
                let bg = Pixel::default();
                for px in dest.pixels_mut() { *px = bg; }
            },
            Previous => if let Some(saved) = self.previous_pixels.take() {
                for (px, &src) in dest.pixels_mut().zip(saved.iter()) { *px = src; }
            },
            Keep | Any => {},
        }
    }

    pub fn new(method: gif::DisposalMethod, left: u16, top: u16, width: u16, height: u16, pixels: ImgRef<'_, Pixel>) -> Self {
        Disposal {
            previous_pixels: match method {
                Previous => Some(pixels.sub_image(left.into(), top.into(), width.into(), height.into()).pixels().collect()),
                _ => None,
            },
            method, left, top, width, height,
        }
    }
}
