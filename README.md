## Please dispose of GIF frames properly

This crate implements GIF disposal method for the [gif crate](https://lib.rs/crates/gif).

The gif crate only exposes raw frame data that is not sufficient
to render animated GIFs properly. GIF requires special composing of frames
which is non-trivial.

## Usage

```rust
let file = File::open("example.gif")?;
let mut decoder = Decoder::new(file);

// Important:
decoder.set(gif::ColorOutput::Indexed);

let mut reader = decoder.read_info()?;

let mut screen = Screen::new_reader(&reader);
while let Some(frame) = reader.read_next_frame()? {
    screen.blit_frame(&frame)?;
    screen.pixels // that's the frame now in RGBA format
}
```

The `screen.pixels` buffer uses [ImgVec](https://lib.rs/crates/imgref) to represent a 2D image.

See `examples/explode.rs` for more.

## Requirements

* Latest stable Rust (1.42+)
