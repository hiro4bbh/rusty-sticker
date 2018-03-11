use std::ops::{Deref,DerefMut};
use std::time::Instant;

extern crate rand;
use rand::Rng;

fn hash_u32(x: u32) -> u32 {
	x*2654435761
}

struct HashMap32(Box<[(u32, u32)]>);

impl HashMap32 {
    fn new(n: usize) -> HashMap32 {
        HashMap32(vec![(0u32, 0u32); n].into_boxed_slice())
    }
    fn inc(&mut self, key: u32) -> (u32, u32) {
        let mask = (self.len() - 1) as usize;
        let mut k = (hash_u32(key) as usize) & mask;
        while self[k].1 > 0 {
            if self[k].0 == key {
                self[k].1 += 1;
                return self[k]
            }
            k = (k + 1) & mask
        }
        self[k] = (key, 1);
        self[k]
    }
}

impl Deref for HashMap32 {
    type Target = [(u32, u32)];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HashMap32 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn run_insert(n: usize, p: usize) {
    let mut rng = rand::thread_rng();
    let mut m = HashMap32::new(n);
    for _ in 0..(n*p/100) {
        m.inc(rng.next_u32()&((n - 1) as u32));
    }
}

fn main() {
    let t = 10000;
    let n = 64*1024;
    let p = 50;
    let start = Instant::now();
    for _ in 0..t {
        run_insert(n, p)
    }
    let elapsed = start.elapsed();
    let elapsed_per_entry = elapsed.checked_div(t as u32).unwrap();
    println!("insert {}% random u32 into {} buckets {} times finished in {}.{}s ({}.{}ms/entry)", p, n, t, elapsed.as_secs(), elapsed.subsec_nanos()/1_000_000, elapsed_per_entry.subsec_nanos()/1_000_000, elapsed_per_entry.subsec_nanos()/1_000%1_000)
}
