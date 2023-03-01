use clap::Parser;
use image::io::Reader as ImageReader;
use std::error::Error;

use image_lsb::{ImageDecoder, ImageEncoder};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
struct Cli {
    /// mode: encode or decode
    mode: String,
    /// The path to the file to read
    path: std::path::PathBuf,
    /// The path to save the file
    save_path: Option<std::path::PathBuf>,
    /// The seed
    #[arg(short = 'S')]
    seed: Option<u64>,
    /// The payload
    #[arg(short = 'P')]
    payload: Option<String>,
}

#[allow(unused_must_use)]
fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    if args.mode.to_lowercase() == "encode" {
        if let None = args.save_path {
            panic!("Save path is required");
        }
        if let None = args.payload {
            panic!("Payload is required");
        }
        if let None = args.seed {
            panic!("Seed is required");
        }

        if let Some(seed) = args.seed {
            let img = ImageReader::open(args.path)?.decode()?;
            let mut encoder = ImageEncoder::new(img, seed);
            encoder.write_str(args.payload.unwrap().as_str());
            encoder.get_back_image().save(args.save_path.unwrap());
        }
    } else if args.mode.to_lowercase() == "decode" {
        if let None = args.seed {
            panic!("Seed is required");
        }

        if let Some(seed) = args.seed {
            let img = ImageReader::open(args.path)?.decode()?;
            let mut decoder = ImageDecoder::new(img, seed);
            println!("Decoded: {}", decoder.read().unwrap());
        }
    } else {
        panic!("Invalid mode");
    }

    Ok(())
}
