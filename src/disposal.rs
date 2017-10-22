
use gif;
use gif::DisposalMethod::*;
use imgref::*;
use subimage::Subimage;
use std::default::Default;

pub struct Disposal<Pixel: Copy> {
    method: gif::DisposalMethod,
    previous_pixels: Option<Vec<Pixel>>,
    left: u16, top: u16,
    width: u16, height: u16,
}

impl<Pixel: Copy> Default for Disposal<Pixel> {
    fn default() -> Self {
        Disposal {
           method: gif::DisposalMethod::Keep,
           previous_pixels: None,
           top: 0, left: 0, width: 0, height: 0,
       }
   }
}

impl<Pixel: Copy> Disposal<Pixel> {
    pub fn dispose(&mut self, pixels: &mut [Pixel], stride: usize, bg_color: Pixel) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        let pixels_iter = pixels.iter_mut().subimage(self.left as usize, self.top as usize, self.width as usize, self.height as usize, stride);
        match self.method {
            Background => for px in pixels_iter { *px = bg_color; },
            Previous => if let Some(saved) = self.previous_pixels.take() {
                for (px, &src) in pixels_iter.zip(saved.iter()) { *px = src; }
            },
            Keep | Any => {},
        }
    }

    pub fn new(method: gif::DisposalMethod, left: u16, top: u16, width: u16, height: u16, pixels: ImgRef<Pixel>) -> Self {
        Disposal {
            previous_pixels: match method {
                Previous => Some(pixels.iter().cloned().subimage(left as usize, top as usize, width as usize, height as usize, pixels.stride()).collect()),
                _ => None,
            },
            method, left, top, width, height,
        }
    }
}
