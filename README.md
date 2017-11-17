## Please dispose of GIF frames properly

This crate implements GIF disposal method for the [gif crate](https://crates.io/crates/gif).

The gif crate only exposes raw frame data that is not sufficient
to render GIFs properly. GIF requires special composing of frames
which is non-trivial.

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
