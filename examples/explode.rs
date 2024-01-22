use std::fs::File;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gif_path = PathBuf::from(std::env::args().nth(1).ok_or("Please specify a GIF file as an argument")?);
    let base_name = gif_path.file_stem().and_then(|s| s.to_str()).ok_or("Invalid filename")?;
    let file = File::open(&gif_path)?;

    let mut gif_opts = gif::DecodeOptions::new();
    gif_opts.set_color_output(gif::ColorOutput::Indexed);
    let mut decoder = gif_opts.read_info(file)?;

    let mut screen = gif_dispose::Screen::new_decoder(&decoder);
    let mut n = 1;
    while let Some(frame) = decoder.read_next_frame()? {
        screen.blit_frame(&frame)?;

        let frame_file = format!("{}-{:04}.png", base_name, n);
        println!("{}", frame_file);
        let (buf, width, height) = screen.pixels_rgba().to_contiguous_buf();
        lodepng::encode32_file(frame_file, &buf, width, height)?;
        n += 1;
    }

    Ok(())
}
