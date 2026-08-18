use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::Rng as _;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Named random streams, one per game system.
///
/// Balatro derives every roll from `pseudoseed(key)` — a separate stream per effect, so drawing
/// a Glass card's break chance never disturbs what the shop stocks. This mirrors that: each key
/// gets its own generator seeded from `(run seed, key)`, and streams advance independently.
///
/// The practical benefit is that a seed keeps meaning something. With one shared stream, adding
/// or removing any RNG consumer anywhere shifts every later roll in the run, so no seed survives
/// an engine change. Here a change only perturbs the stream it touches.
///
/// Keys mirror Balatro's where an equivalent exists (`glass`, `misprint`, `idol`, `wheel`, …).
/// Per-ante streams are spelled `keyed("idol", ante)`, matching `pseudoseed('idol'..ante)`.
///
/// Current keys, by area:
///
/// - **cards**: `shuffle`, `front`, `erratic`, `edition_deck`, `edition_generic`
/// - **scoring**: `glass`, `lucky_mult`, `lucky_money`, `bloodstone`, `misprint`, `business`,
///   `parking`, `space`, `8ball`, `seance`, `sixth`, `gros_michel`, `cavendish`
/// - **shop**: `shop_slot`, `booster_pool`, `rarity`, `joker_pool`, `tarot`, `planet`,
///   `spe_card`, `voucher`, `illusion`, `eternal`, `perishable`, `rental`
/// - **blinds**: `boss`, `hook`, `wheel`, `cerulean_bell`, `crimson_heart`, `amber_acorn`
/// - **tags**: `tag`, `tag_joker`, `orbital`, `top`
/// - **consumables**: `wheel_of_fortune`, `aura`, `editionless`, `ankh_choice`, `wraith`,
///   `sigil`, `ouija`, `immolate`, `soul_`, `familiar_create`, `grim_create`,
///   `incantation_create`
/// - **jokers**: `madness`, `invisible`, `perkeo`, `to_do`, `cert_fr`, `certsl`, `halu`
/// - **per ante**: `idol`, `mail`, `cas`, `anc`, `stdset`, `stdseal`, `stdsealtype`
///
/// Adding a key is free; reusing an existing one couples the two effects, so prefer a new one.
pub struct Rng {
    run_seed: u64,
    streams: HashMap<String, SmallRng>,
}

/// Build the per-ante (or per-round) form of a stream key, e.g. `keyed("idol", 3) == "idol3"`.
pub fn keyed(key: &str, n: u32) -> String {
    format!("{}{}", key, n)
}

impl Rng {
    pub fn new(seed: &str) -> Self {
        let run_seed = if seed.is_empty() {
            42u64
        } else {
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            hasher.finish()
        };
        Self {
            run_seed,
            streams: HashMap::new(),
        }
    }

    /// The stream for `key`, created on first use. Seeded from the run seed mixed with the key,
    /// so two runs with the same seed produce the same stream and different keys never collide.
    fn stream(&mut self, key: &str) -> &mut SmallRng {
        let run_seed = self.run_seed;
        self.streams.entry(key.to_string()).or_insert_with(|| {
            let mut hasher = DefaultHasher::new();
            run_seed.hash(&mut hasher);
            key.hash(&mut hasher);
            SmallRng::seed_from_u64(hasher.finish())
        })
    }

    pub fn next_u64(&mut self, key: &str) -> u64 {
        self.stream(key).next_u64()
    }

    pub fn next_f64(&mut self, key: &str) -> f64 {
        self.stream(key).gen()
    }

    /// Roll a listed probability. `probability` is already scaled by the caller for
    /// Oops! All 6s, which doubles every listed chance.
    pub fn next_bool_prob(&mut self, key: &str, probability: f64) -> bool {
        self.stream(key).gen::<f64>() < probability
    }

    pub fn range_u32(&mut self, key: &str, min: u32, max: u32) -> u32 {
        self.stream(key).gen_range(min..=max)
    }

    pub fn range_usize(&mut self, key: &str, min: usize, max: usize) -> usize {
        self.stream(key).gen_range(min..=max)
    }

    pub fn shuffle<T>(&mut self, key: &str, slice: &mut [T]) {
        slice.shuffle(self.stream(key));
    }

    pub fn choose<'a, T>(&mut self, key: &str, slice: &'a [T]) -> Option<&'a T> {
        if slice.is_empty() {
            return None;
        }
        let idx = self.range_usize(key, 0, slice.len() - 1);
        slice.get(idx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_reproduces_the_same_stream() {
        let mut a = Rng::new("SEED");
        let mut b = Rng::new("SEED");
        let xs: Vec<u64> = (0..20).map(|_| a.next_u64("glass")).collect();
        let ys: Vec<u64> = (0..20).map(|_| b.next_u64("glass")).collect();
        assert_eq!(xs, ys);
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = Rng::new("SEED_A");
        let mut b = Rng::new("SEED_B");
        assert_ne!(a.next_u64("glass"), b.next_u64("glass"));
    }

    #[test]
    fn streams_are_independent() {
        // Draining one stream must not shift another - this is the whole point of keying.
        let mut a = Rng::new("SEED");
        let mut b = Rng::new("SEED");
        for _ in 0..50 {
            a.next_u64("shop");
        }
        assert_eq!(a.next_u64("glass"), b.next_u64("glass"));
    }

    #[test]
    fn different_keys_produce_different_sequences() {
        let mut r = Rng::new("SEED");
        let xs: Vec<u64> = (0..10).map(|_| r.next_u64("glass")).collect();
        let ys: Vec<u64> = (0..10).map(|_| r.next_u64("wheel")).collect();
        assert_ne!(xs, ys);
    }

    #[test]
    fn per_ante_keys_are_separate_streams() {
        let mut r = Rng::new("SEED");
        let a1: Vec<u64> = (0..10).map(|_| r.next_u64(&keyed("idol", 1))).collect();
        let a2: Vec<u64> = (0..10).map(|_| r.next_u64(&keyed("idol", 2))).collect();
        assert_ne!(a1, a2);
    }

    #[test]
    fn shuffle_is_reproducible_and_stream_scoped() {
        let mut a = Rng::new("SEED");
        let mut b = Rng::new("SEED");
        let mut xs: Vec<u32> = (0..30).collect();
        let mut ys: Vec<u32> = (0..30).collect();
        for _ in 0..10 {
            b.next_u64("unrelated");
        }
        a.shuffle("shuffle", &mut xs);
        b.shuffle("shuffle", &mut ys);
        assert_eq!(xs, ys);
    }
}
