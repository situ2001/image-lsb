use std::collections::HashSet;

use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

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

        let mut x = self.rng.gen_range(0..=self.width);
        let mut y = self.rng.gen_range(0..=self.height);
        while self.track.contains(&(x, y)) {
            x = self.rng.gen_range(0..=self.width);
            y = self.rng.gen_range(0..=self.height);
        }
        self.track.insert((x, y));
        (x, y)
    }
}

#[cfg(test)]
mod tests {
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
}
