use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng as _;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct Rng {
    rng: SmallRng,
}

impl Rng {
    pub fn new(seed: &str) -> Self {
        let numeric_seed = if seed.is_empty() {
            42u64
        } else {
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            hasher.finish()
        };
        Self {
            rng: SmallRng::seed_from_u64(numeric_seed),
        }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }

    pub fn next_f64(&mut self) -> f64 {
        self.rng.gen()
    }

    pub fn next_bool_prob(&mut self, probability: f64) -> bool {
        self.rng.gen::<f64>() < probability
    }

    pub fn range_u32(&mut self, min: u32, max: u32) -> u32 {
        self.rng.gen_range(min..=max)
    }

    pub fn range_usize(&mut self, min: usize, max: usize) -> usize {
        self.rng.gen_range(min..=max)
    }

    pub fn shuffle<T>(&mut self, slice: &mut [T]) {
        slice.shuffle(&mut self.rng);
    }

    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            return None;
        }
        let idx = self.rng.gen_range(0..slice.len());
        Some(&slice[idx])
    }

    pub fn choose_index(&mut self, len: usize) -> Option<usize> {
        if len == 0 {
            None
        } else {
            Some(self.rng.gen_range(0..len))
        }
    }
}
