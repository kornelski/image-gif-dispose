use gif::*;
use std::fs::File;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gif_path = PathBuf::from(std::env::args().nth(1).ok_or("Please specify a GIF file as an argument")?);
    let base_name = gif_path.file_stem().and_then(|s| s.to_str()).ok_or("Invalid filename")?;

    let mut decoder = gif::Decoder::new(File::open(&gif_path)?);
    decoder.set(gif::ColorOutput::Indexed);
    let mut reader = decoder.read_info()?;

    let mut screen = gif_dispose::Screen::new_reader(&reader);
    let mut n = 1;
    while let Some(frame) = reader.read_next_frame()? {
        screen.blit_frame(&frame)?;

        let frame_file = format!("{}-{:04}.png", base_name, n);
        println!("{}", frame_file);
        lodepng::encode32_file(frame_file, &screen.pixels.buf(), screen.pixels.width(), screen.pixels.height())?;
        n += 1;
    }

    Ok(())
}
