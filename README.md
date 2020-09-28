## Please dispose of GIF frames properly

This crate implements GIF disposal method for the [gif crate](https://lib.rs/crates/gif).

The gif crate only exposes raw frame data that is not sufficient
to render animated GIFs properly. GIF requires special composing of frames
which is non-trivial.

## Usage

```rust
let file = File::open("example.gif")?;

let mut gif_opts = gif::DecodeOptions::new();
// Important:
gif_opts.set_color_output(gif::ColorOutput::Indexed);

let mut decoder = gif_opts.read_info(file)?;
let mut screen = gif_dispose::Screen::new_decoder(&decoder);

while let Some(frame) = decoder.read_next_frame()? {
    screen.blit_frame(&frame)?;
    screen.pixels // that's the frame now in RGBA format
}
```

The `screen.pixels` buffer uses [ImgVec](https://lib.rs/crates/imgref) to represent a 2D image.

See `examples/explode.rs` for more.

## Requirements

* Latest stable Rust (1.45+)
