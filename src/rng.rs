use std::collections::HashSet;

const RNG_STATE_SIZE: usize = 16;

pub trait LinearRNG: Clone {
    fn next_u32(&mut self) -> u32;
    fn next_f64(&mut self, range: f64) -> f64;
    fn skip(&mut self, amount: usize);
}

#[derive(Clone)]
pub struct RNG {
    random_poly: u32,
    index: usize,
    state: [u32; RNG_STATE_SIZE]
}

pub struct PrecomputedRNG {
    values: Vec<u32>
}

#[derive(Clone)]
pub struct LinearPrecomputedRNG<'a> {
    precomputed_rng: &'a PrecomputedRNG,
    position: usize
}

impl RNG {
    pub fn new(mut seed: u32, seeds_15bit: bool, seeds_signed: bool, use_old_random_poly: bool) -> RNG {
        let mut rng: RNG = RNG {
            random_poly: if use_old_random_poly { 0xda442d20 } else { 0xda442d24 },
            index: 0,
            state: [0; RNG_STATE_SIZE]
        };

        // Generate initial state
        rng.index = 0;
        if seeds_15bit {
            for i in 0..RNG_STATE_SIZE {
                seed = u32::wrapping_shr(u32::wrapping_add(u32::wrapping_mul(seed, 0x343fd), 0x269ec3), 16) & 0x7fff;
                rng.state[i] = seed;
            }
        } else if seeds_signed {
            let mut signed_seed = seed as i32;
            for i in 0..RNG_STATE_SIZE {
                signed_seed = i32::wrapping_shr(i32::wrapping_add(i32::wrapping_mul(signed_seed, 0x343fd), 0x269ec3), 16) & 0x7fffffff;
                rng.state[i] = signed_seed as u32;
            }
        } else {
            let mut signed_seed = seed as i32;
            signed_seed = i32::wrapping_shr(i32::wrapping_add(i32::wrapping_mul(signed_seed, 0x343fd), 0x269ec3), 16) & 0x7fffffff;
            rng.state[0] = signed_seed as u32;
            for i in 1..RNG_STATE_SIZE {
               signed_seed = u32::wrapping_shr(i32::wrapping_add(i32::wrapping_mul(signed_seed, 0x343fd), 0x269ec3) as u32, 16) as i32;
               rng.state[i] = signed_seed as u32;
            }
        }

        rng
    }

    pub fn precompute(&mut self, num: usize) -> PrecomputedRNG {
        let mut values: Vec<u32> = Vec::with_capacity(num);
        for _ in 0..num {
            values.push(self.next_u32());
        }
        PrecomputedRNG {
            values
        }
    }

    pub fn calculate_unique_seeds(seeds_15bit: bool, seeds_signed: bool) -> Vec<u32> {
        let unique_state_count: usize = if seeds_15bit { 32768 } else { 65536 }; 
        let mut unique_seeds_list: Vec<u32> = Vec::with_capacity(unique_state_count);
        let mut unique_states: HashSet<u32> = HashSet::with_capacity(unique_state_count);

        if seeds_15bit {
            let mut curr_seed: u32 = 0;
            while unique_states.len() < unique_state_count {
                let state: u32 = u32::wrapping_shr(u32::wrapping_add(u32::wrapping_mul(curr_seed, 0x343fd), 0x269ec3), 16) & 0x7fff;
                if unique_states.insert(state) {
                    unique_seeds_list.push(curr_seed);
                }
                curr_seed += 1;
            }
        } else if seeds_signed {
            let mut curr_seed: u32 = 0;
            while unique_states.len() < unique_state_count {
                let state: u32 = (i32::wrapping_shr(i32::wrapping_add(i32::wrapping_mul(curr_seed as i32, 0x343fd), 0x269ec3), 16) & 0x7fffffff) as u32;
                if unique_states.insert(state) {
                    unique_seeds_list.push(curr_seed);
                }
                curr_seed += 1;
            }
        } else {
            let mut curr_seed: u32 = 0;
            while unique_states.len() < unique_state_count {
                let state: u32 = (i32::wrapping_shr(i32::wrapping_add(i32::wrapping_mul(curr_seed as i32, 0x343fd), 0x269ec3), 16) & 0x7fffffff) as u32;
                if unique_states.insert(state) {
                    unique_seeds_list.push(curr_seed);
                }
                curr_seed += 1;
            }
        }

        unique_seeds_list
    }
}
impl LinearRNG for RNG {
    fn next_u32(&mut self) -> u32 {
        let mut a: u32 = self.state[self.index];
        let mut b: u32 = self.state[(self.index + 13) & 15];
        let c: u32 = a ^ b ^ u32::wrapping_shl(a, 16) ^ u32::wrapping_shl(b, 15);
        b = self.state[(self.index + 9) & 15];
        b ^= u32::wrapping_shr(b, 11);
        a = c ^ b;
        self.state[self.index] = a;
        let d: u32 = a ^ (u32::wrapping_shl(a, 5) & self.random_poly);
        self.index = (self.index + 15) & 15;
        a = self.state[self.index];
        self.state[self.index] = a ^ c ^ d ^ u32::wrapping_shl(a, 2) ^ u32::wrapping_shl(c, 18) ^ u32::wrapping_shl(b, 28);
        self.state[self.index]
    }
    fn next_f64(&mut self, range: f64) -> f64 {
        return (self.next_u32() as f64) * 2.3283064365386963e-10 * range;
    }
    fn skip(&mut self, amount: usize) {
        for _ in 0..amount {
            _ = self.next_u32();
        }
    }
}

impl PrecomputedRNG {
    pub fn get_u32(&self, position: usize) -> u32 {
        self.values[position]
    }
    pub fn get_f64(&self, range: f64, position: usize) -> f64 {
        return (self.values[position] as f64) * 2.3283064365386963e-10 * range;
    }
    pub fn raw(&self) -> &Vec<u32> {
        &self.values
    }
}

impl LinearPrecomputedRNG<'_> {
    pub fn new<'a>(precomputed_rng: &'a PrecomputedRNG, position: usize) -> LinearPrecomputedRNG<'a> {
        LinearPrecomputedRNG {
            precomputed_rng,
            position
        }
    }
}
impl LinearRNG for LinearPrecomputedRNG<'_> {
    fn next_u32(&mut self) -> u32 {
        let value = self.precomputed_rng.get_u32(self.position);
        self.position += 1;
        value
    }
    fn next_f64(&mut self, range: f64) -> f64 {
        let value = self.precomputed_rng.get_f64(range, self.position);
        self.position += 1;
        value
    }
    fn skip(&mut self, amount: usize) {
        self.position += amount;
    }
}