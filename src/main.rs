use bitvec::{macros::internal::funty::Fundamental, prelude::*};
use image::{io::Reader as ImageReader, GenericImage, GenericImageView};
use regex::Regex;
use std::error::Error;

use lsb_image::f;

fn main() -> Result<(), Box<dyn Error>> {
    println!("{}", f());

    let args = std::env::args().collect::<Vec<_>>();

    if args.len() == 2 {
        println!("usage: main <input_path> <payload_string>");
        return Ok(());
    }

    dbg!(args);
    // let input_path = &args[1];
    // let payload_str = &args[2];

    let mut img = ImageReader::open("./input.png")?.decode()?;

    // get width and height
    let (width, height) = img.dimensions();
    println!("width: {}, height: {}", width, height);
    let mut x = 0;
    let mut y = 0;
    let payload_str = "situ2001";

    // first fill metadata
    let metadata_str = format!("{{{{{}}}}}", payload_str.len());
    println!("{}", metadata_str);
    // concat it to payload string
    let payload_str = format!("{}{}", metadata_str, payload_str);

    // fill the payload string
    for ch in payload_str.as_bytes() {
        println!("Current char: {}", ch.as_char().unwrap());
        let bits = ch.view_bits::<Msb0>();
        for bit in bits {
            // println!("bit: {}", if bit == true { 1 } else { 0 });
            let bit_to_be_filled = if bit == true { 1 } else { 0 };
            let mut new_pixel = img.get_pixel(x, y);
            new_pixel.0[0] = new_pixel.0[0] & 0b11111110 | bit_to_be_filled;
            img.put_pixel(x, y, new_pixel);
            // TODO refactor with an abstract next() method, e.g, the next x y are generated from a random sequence with a specific seed
            if x == width - 1 {
                x = 0;
                y += 1;
            } else {
                x += 1;
            }
        }
        println!("{} => {:?}", ch, bits);
    }
    // save the new image
    img.save("./output.png")?;

    {
        // decode the image
        let img = ImageReader::open("./output.png")?.decode()?;
        let (width, height) = img.dimensions();
        println!("width: {}, height: {}", width, height);

        let mut x = 0;
        let mut y = 0;
        let payload_string_len: usize;

        // read string like {{len}} from image
        let mut metadata_string = String::new();
        let mut current_char_bits: Vec<u8> = Vec::new();
        loop {
            let pixel = img.get_pixel(x, y);
            let bit = pixel.0[0] & 0b00000001;
            // println!("bit: {}", bit);
            current_char_bits.push(bit);
            if current_char_bits.len() >= 8 {
                let collected_char = current_char_bits
                    .iter()
                    .fold(0, |acc, &bit| (acc << 1) | bit);
                // convert to char and append to string
                metadata_string.push(collected_char as char);
                // println!("Collected: {}", collected_char as char);
                current_char_bits.clear();
            }
            if x == width - 1 {
                x = 0;
                y += 1;
            } else {
                x += 1;
            }
            if metadata_string.len() == 2 && metadata_string != "{{" {
                println!("Invalid metadata");
                return Ok(());
            }
            // regex match {{.*}}
            if metadata_string.len() >= 4 && metadata_string.ends_with("}}") {
                let re = Regex::new(r"\{\{(\d+)\}\}").unwrap();
                if let Some(caps) = re.captures(&metadata_string) {
                    payload_string_len = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
                    println!("Payload len: {}", payload_string_len);
                    break;
                }
            }
        }

        let mut payload = String::new();
        let mut current_char_bits: Vec<u8> = Vec::new();
        for _ in 0..payload_string_len * 8 {
            let pixel = img.get_pixel(x, y);
            let bit = pixel.0[0] & 0b00000001;
            // println!("bit: {}", bit);
            current_char_bits.push(bit);
            if current_char_bits.len() >= 8 {
                let collected_char = current_char_bits
                    .iter()
                    .fold(0, |acc, &bit| (acc << 1) | bit);
                // convert to char and append to string
                payload.push(collected_char as char);
                current_char_bits.clear();
            }
            if x == width - 1 {
                x = 0;
                y += 1;
            } else {
                x += 1;
            }
        }
        println!("Decoded: {}", payload);
    }

    Ok(())
}
