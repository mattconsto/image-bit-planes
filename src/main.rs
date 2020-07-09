use image::{GenericImageView, ImageBuffer, Pixel, Rgba, RgbaImage};

fn main() -> Result<(), String> {
    let matches = clap::App::new("image-bit-planes")
        .version("0.1")
        .author("Matt Consto. <matt@consto.uk>")
        .about("Split an image into bit planes")
        .arg(
            clap::Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::with_name("OUTPUT")
                .help("Sets the output file to use")
                .required(true)
                .index(2),
        )
        .arg(
            clap::Arg::with_name("FLIP_X")
                .long("flip-x")
                .help("Flip the image in the X-axis"),
        )
        .arg(
            clap::Arg::with_name("FLIP_Y")
                .long("flip-y")
                .help("Flip the image in the Y-axis"),
        )
        .arg(
            clap::Arg::with_name("FOREGROUND")
                .long("foreground")
                .help("Specify the foreground color")
                .default_value("ffffff"),
        )
        .arg(
            clap::Arg::with_name("BACKGROUND")
                .long("background")
                .help("Specify the background color")
                .default_value("000000"),
        )
        .arg(
            clap::Arg::with_name("VERBOSE")
                .short("v")
                .multiple(true)
                .help("Increases the level of debugging information"),
        )
        .get_matches();

    // Handle unwrapping/converting clap output
    let input_filename = matches.value_of("INPUT").ok_or("Invalid input file")?;
    let output_filename = matches.value_of("OUTPUT").ok_or("Invalid output file")?;
    let flip_x = matches.is_present("FLIP_X");
    let flip_y = matches.is_present("FLIP_Y");
    let foreground = hex_to_rgb(matches.value_of("FOREGROUND").unwrap_or(""))
        .ok_or("Invalid foreground color")?;
    let background = hex_to_rgb(matches.value_of("BACKGROUND").unwrap_or(""))
        .ok_or("Invalid background color")?;
    let verbose = matches.is_present("VERBOSE");

    if verbose {
        println!("Opening {} for reading", input_filename)
    }

    // Open file
    let input = image::open(input_filename).or(Err("Failed to open input file"))?;
    let channel_count = input.color().channel_count() as u32;
    let bits_per_subpixel = input.color().bits_per_pixel() as u32 / channel_count;
    let (width, height) = input.dimensions();

    if verbose {
        println!(
            "{} x {}, {} channels, {} bpp",
            width, height, channel_count, bits_per_subpixel
        )
    }

    let mut output: RgbaImage = ImageBuffer::new(width * bits_per_subpixel, height * channel_count);

    // Do the work
    for (x, y, pixel) in input.pixels() {
        for i in 0..channel_count {
            for j in 0..bits_per_subpixel {
                output.put_pixel(
                    if flip_x {
                        x + (bits_per_subpixel - j - 1) * width
                    } else {
                        x + j * width
                    },
                    if flip_y {
                        y + (channel_count - i - 1) * height
                    } else {
                        y + i * height
                    },
                    if ((pixel.channels()[i as usize]) & (1 << j)) != 0 {
                        foreground
                    } else {
                        background
                    },
                );
            }
        }
    }

    if verbose {
        println!("Opening {} for writing", output_filename)
    }

    // Write file
    output
        .save(output_filename)
        .or(Err("Failed to write output file"))?;

    if verbose {
        println!("Complete")
    }

    Ok(())
}

/// Convert a String to image::Rgba
fn hex_to_rgb(h: &str) -> Option<Rgba<u8>> {
    // Handle 3 or 4 character hex codes such as fff
    let bytes = hex::decode(if h.trim().len() == 3 || h.trim().len() == 4 {
        h.trim()
            .chars()
            .map(|c| c.to_string().repeat(2))
            .collect::<Vec<String>>()
            .join("")
    } else {
        h.trim().to_owned()
    })
    .ok()?;

    if bytes.len() > 4 {
        eprintln!("{}", "Truncating colors to 8 bits per channel");
    }

    // Use the first byte per channel, assumes big-endianness
    if bytes.len() % 3 == 0 && bytes.len() >= 3 {
        let size = bytes.len() / 3;
        Some(Rgba([bytes[0], bytes[size], bytes[2 * size], 255]))
    } else if bytes.len() % 4 == 0 && bytes.len() >= 4 {
        let size = bytes.len() / 4;
        Some(Rgba([
            bytes[0],
            bytes[size],
            bytes[2 * size],
            bytes[3 * size],
        ]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::hex_to_rgb;
    use image::Rgba;

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(hex_to_rgb(""), None);
        assert_eq!(hex_to_rgb(" "), None);
        assert_eq!(hex_to_rgb("invalid"), None);
        assert_eq!(hex_to_rgb("000"), Some(Rgba([0, 0, 0, 255])));
        assert_eq!(hex_to_rgb("ffffff"), Some(Rgba([255, 255, 255, 255])));
        assert_eq!(hex_to_rgb("11223344"), Some(Rgba([17, 34, 51, 68])));
    }
}
