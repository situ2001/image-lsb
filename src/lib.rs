#![allow(dead_code)]

use std::collections::HashSet;

use bitvec::prelude::*;

use image::GenericImage;
use image::GenericImageView;
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use regex::Regex;

pub struct PixelGenerator {
    rng: StdRng,
    width: u32,
    height: u32,
    track: HashSet<(u32, u32)>,
}

// will generate unused random pixels
impl PixelGenerator {
    pub fn new(seed: u64, width: u32, height: u32) -> Self {
        PixelGenerator {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
            width,
            height,
            track: HashSet::new(),
        }
    }

    pub fn next(&mut self) -> (u32, u32) {
        if self.track.len() == (self.width * self.height) as usize {
            // throw Error
            panic!("PixelGenerator has no more pixels to generate");
        }

        let mut x = self.rng.gen_range(0..self.width);
        let mut y = self.rng.gen_range(0..self.height);
        while self.track.contains(&(x, y)) {
            x = self.rng.gen_range(0..self.width);
            y = self.rng.gen_range(0..self.height);
        }
        self.track.insert((x, y));
        (x, y)
    }
}

pub struct ImageEncoder {
    img: image::DynamicImage,
    gen: PixelGenerator,
    bit_written: usize,
}

impl ImageEncoder {
    pub fn new(img: image::DynamicImage, seed: u64) -> Self {
        let img = img;
        let (width, height) = img.dimensions();
        let gen = PixelGenerator::new(seed, width, height);
        ImageEncoder {
            img,
            gen,
            bit_written: 0,
        }
    }

    fn check_available_space(&self, s: &str) -> bool {
        let (width, height) = self.img.dimensions();
        let payload_len = s.len() * 8;
        let available_space = (width * height) as usize - self.bit_written;
        available_space >= payload_len
    }

    fn write_bit(&mut self, bit: bool) {
        let (x, y) = self.gen.next();
        let mut new_pixel = self.img.get_pixel(x, y);
        new_pixel.0[0] = new_pixel.0[0] & 0b11111110 | if bit { 1 } else { 0 };
        self.img.put_pixel(x, y, new_pixel);
        self.bit_written += 1;
    }

    fn write_metadata(&mut self, s: &str) {
        // TODO make it a JSON struct
        let metadata_str = format!("{{{{{}}}}}", s.len());
        if !self.check_available_space(&metadata_str) {
            panic!("Not enough space to write metadata");
        }
        dbg!(&metadata_str);
        for ch in metadata_str.as_bytes() {
            let bits = ch.view_bits::<Msb0>();
            for bit in bits {
                self.write_bit(bit == true);
            }
        }
    }

    pub fn write_str(&mut self, s: &str) {
        if !self.check_available_space(s) {
            panic!("Not enough space to write payload");
        }

        self.write_metadata(s);
        for ch in s.as_bytes() {
            let bits = ch.view_bits::<Msb0>();
            for bit in bits {
                self.write_bit(bit == true);
            }
        }
    }

    pub fn get_back_image(self) -> image::DynamicImage {
        self.img
    }
}

pub struct ImageDecoder {
    img: image::DynamicImage,
    gen: PixelGenerator,
}

impl ImageDecoder {
    pub fn new(img: image::DynamicImage, seed: u64) -> Self {
        let img = img;
        let (width, height) = img.dimensions();
        let gen = PixelGenerator::new(seed, width, height);
        ImageDecoder { img, gen }
    }

    fn read_bit(&mut self) -> bool {
        let (x, y) = self.gen.next();
        let pixel = self.img.get_pixel(x, y);
        pixel.0[0] & 0b00000001 == 1
    }

    fn read_metadata(&mut self) -> Option<usize> {
        let mut metadata_string = String::new();
        let mut current_char_bits: Vec<u8> = Vec::new();
        let payload_string_len: usize;
        loop {
            let bit = self.read_bit();
            current_char_bits.push(if bit { 1 } else { 0 });
            if current_char_bits.len() >= 8 {
                let collected_char = current_char_bits
                    .iter()
                    .fold(0, |acc, &bit| (acc << 1) | bit);
                metadata_string.push(collected_char as char);
                current_char_bits.clear();
            }
            if metadata_string.len() == 2 && metadata_string != "{{" {
                return None;
            }
            if metadata_string.len() >= 4 && metadata_string.ends_with("}}") {
                let re = Regex::new(r"\{\{(\d+)\}\}").unwrap();
                if let Some(caps) = re.captures(&metadata_string) {
                    payload_string_len = caps.get(1).unwrap().as_str().parse::<usize>().unwrap();
                    //println!("Payload len: {}", payload_string_len);
                    break;
                }
            }
        }
        Some(payload_string_len)
    }

    fn read_str(&mut self, str_len: usize) -> String {
        let mut payload_string = String::new();
        let mut current_char_bits: Vec<u8> = Vec::new();
        for _ in 0..str_len * 8 {
            let bit = self.read_bit();
            current_char_bits.push(if bit { 1 } else { 0 });
            if current_char_bits.len() >= 8 {
                let collected_char = current_char_bits
                    .iter()
                    .fold(0, |acc, &bit| (acc << 1) | bit);
                payload_string.push(collected_char as char);
                current_char_bits.clear();
            }
        }
        payload_string
    }

    pub fn read(&mut self) -> Option<String> {
        if let Some(payload_len) = self.read_metadata() {
            Some(self.read_str(payload_len))
        } else {
            None
        }
    }

    pub fn read_all(&mut self) -> Option<String> {
        let mut s = String::new();
        let has_any_s = false;
        while let Some(payload_len) = self.read_metadata() {
            s.push_str(&self.read_str(payload_len));
        }
        if has_any_s == true {
            Some(s)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use image::{DynamicImage, RgbImage};

    use super::*;

    #[test]
    fn test_pixel_generator() {
        let mut gen = PixelGenerator::new(114514, 120, 240);
        // should have 120 * 240 pixels after 120 * 240 next() calls
        for _ in 0..(120 * 240) {
            gen.next();
        }
        assert_eq!(gen.track.len(), 120 * 240);
        // should generate same sequence of pixels with the same seed
        let mut gen = PixelGenerator::new(114514, 120, 240);
        let mut gen2 = PixelGenerator::new(114514, 120, 240);
        for _ in 0..(120 * 240) {
            assert_eq!(gen.next(), gen2.next());
        }
    }

    #[test]
    #[should_panic]
    fn test_pixel_generator_excess_next_call() {
        let mut gen = PixelGenerator::new(114514, 120, 240);
        // should panic after 120 * 240 + 1 next() calls
        for _ in 0..(120 * 240 + 1) {
            gen.next();
        }
    }

    fn generate_test_image(width: u32, height: u32) -> DynamicImage {
        let image = RgbImage::new(width, height);
        DynamicImage::ImageRgb8(image)
    }

    #[test]
    fn test_image_encoder_decoder_single_write() -> Result<(), Box<dyn Error>> {
        let img = generate_test_image(100, 100);
        let seed = 114514;
        let s = "situ2001, 114514 1919810";

        let mut encoder = ImageEncoder::new(img, seed);
        encoder.write_str(s);

        let mut decoder = ImageDecoder::new(encoder.img, seed);
        assert_eq!(decoder.read().unwrap(), s);

        Ok(())
    }

    #[test]
    fn test_image_encoder_decoder_multiple_write() -> Result<(), Box<dyn Error>> {
        let img = generate_test_image(100, 100);
        let seed = 114514;
        let s = "situ2001, 114514 1919810";

        let mut encoder = ImageEncoder::new(img, seed);
        encoder.write_str(s);
        encoder.write_str(s);
        encoder.write_str(s);
        encoder.write_str(s);
        encoder.write_str(s);

        let mut decoder = ImageDecoder::new(encoder.img, seed);
        assert_eq!(decoder.read().unwrap(), s);
        assert_eq!(decoder.read().unwrap(), s);
        assert_eq!(decoder.read().unwrap(), s);
        assert_eq!(decoder.read().unwrap(), s);
        assert_eq!(decoder.read().unwrap(), s);

        Ok(())
    }

    #[test]
    fn test_image_read_times_more_than_write() {
        let img = generate_test_image(100, 100);
        let seed = 1919810;
        let s = "situ2001 114514 1919810";

        let mut encoder = ImageEncoder::new(img, seed);
        encoder.write_str(s);

        let mut decoder = ImageDecoder::new(encoder.img, seed);
        assert_eq!(decoder.read(), Some(String::from(s)));
        assert_eq!(decoder.read(), None);
        assert_eq!(decoder.read(), None);
    }

    #[test]
    #[should_panic]
    fn test_image_excess_write() {
        let img = generate_test_image(10, 10);
        let seed = 1;
        let s = "situ2001 114514 1919810";

        let mut encoder = ImageEncoder::new(img, seed);
        encoder.write_str(s);
    }

    #[test]
    #[should_panic]
    fn test_image_excess_write_metadata() {
        let img = generate_test_image(4, 4);
        let seed = 1;
        let s = "situ2001 114514 1919810";

        let mut encoder = ImageEncoder::new(img, seed);
        encoder.write_str(s)
    }
}
