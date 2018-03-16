#![allow(non_snake_case)]

use std::env;
use std::ops::{Deref,DerefMut};
use std::process;
use std::time::Instant;

extern crate rand;
use rand::Rng;

fn hash_u32(x: u32) -> u32 {
    let mut h = (x + 1) as u64;
    h ^= h>>16;
    h = (h*0x85ebca6b)&0xffffffff;
    h ^= h>>13;
    h = (h*0xc2b2ae35)&0xffffffff;
    (h^(h>>16)) as u32
}

fn sort_largests_with_heap(kcs: &mut [(u32, u32)], K: usize) {
    // This implementation comes from http://web.archive.org/web/20140807181610/http://fallabs.com/blog-ja/promenade.cgi?id=104.
    // Strategy:
    //   Basically, this is an in-place heap sort.
    //   Retain the heap at the first whose size is at most K for efficiently sorting the K largest counts.
    let K = K.min(kcs.len());
    // First, the only first entry is automatically in the heap.
    let mut cur = 1;
    // The first K entries are inserted into the heap.
    // Any entry in the heap satisfies that the entry is "NOT LARGER" than any descendants of it.
    // The order in the heap are reversed at the third step.
    // The heap structure is retained as follows:
    //   [0] -> [1] -> [3] -> ...
    //              -> [4] -> ...
    //       -> [2] -> [5] -> ...
    //              -> [6] -> ...
    // Hence, prev(k) = (k - 1)/2, left(k) = 2*k + 1, and right(k) = 2*k + 2.
    while cur < K {
        // Insert the cur-th entry into the heap.
        let mut cidx = cur;
        while cidx > 0 {
            let pidx = (cidx - 1) / 2; // prev(cidx)
            if !(kcs[pidx].1 > kcs[cidx].1) {
                break
            }
            // Swap the current entry with the parent one, because the current one is smaller.
            kcs.swap(cidx, pidx);
            // Perform this recursively at the parent entry.
            cidx = pidx
        }
        cur += 1
    }
    // Second, the remain entries are inserted into the heap as keeping the heap size is k.
    while cur < kcs.len() {
        // Insert the current entry if it is larger than the smallest one in the heap.
        if kcs[cur].1 > kcs[0].1 {
            // Procedure A: Insert the current entry into the size K heap.
            // Swap the current entry and the smallest one in the heap.
            kcs.swap(0, cur);
            // Insert the current entry in the heap.
            let (mut pidx, bot) = (0, K/2);
            while pidx < bot {
                // Take the smaller child as the current entry.
                let mut cidx = 2*pidx + 1; // left(cidx)
                if cidx < K - 1 && kcs[cidx].1 > kcs[cidx+1].1 {
                    cidx += 1;
                }
                if kcs[cidx].1 > kcs[pidx].1 {
                    break
                }
                // Swap the current entry with the selected child, because the current one is larger.
                kcs.swap(pidx, cidx);
                // Perform this recursively at the child entry.
                pidx = cidx;
            }
        }
        cur += 1;
    }
    // Third, the entries in the heap are reversed.
    // This is achieved by shrinking the heap one by one:
    //   Taking the largest entry as the current one, insert it to the heap.
    // Hence, the largest entry is being selected as the current one as pushing down the smaller entries.
    cur = K - 1;
    while cur > 0 {
        // Apply procedure A in the size cur heap.
        kcs.swap(0, cur);
        let (mut pidx, bot) = (0, cur/2);
        while pidx < bot {
            let mut cidx = 2*pidx + 1;
            if cidx < cur-1 && kcs[cidx].1 > kcs[cidx+1].1 {
                cidx += 1;
            }
            if kcs[cidx].1 > kcs[pidx].1 {
                break
            }
            kcs.swap(pidx, cidx);
            pidx = cidx;
        }
        cur -= 1;
    }
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

fn run_sort_largests_with_heap(n: usize, p: usize, K: usize) {
    let mut rng = rand::thread_rng();
    // The keys are ignored.
    let mut kcs = vec![(0u32, 0u32); n];
    let kcs = kcs.as_mut_slice();
    for _ in 0..(n*p/100) {
        kcs[(rng.next_u32() as usize)%n].1 = rng.next_u32()
    }
    sort_largests_with_heap(kcs, K)
}

fn run_hashmap32_insert(n: usize, p: usize) {
    let mut rng = rand::thread_rng();
    let mut m = HashMap32::new(n);
    for _ in 0..(n*p/100) {
        m.inc(rng.next_u32());
    }
}

macro_rules! measure_time {
    ($title:expr, $t:expr, $fn:expr) => ({
        let start = Instant::now();
        for _ in 0..($t) {
            $fn
        };
        let elapsed = start.elapsed();
        let elapsed_per_entry = elapsed.checked_div($t as u32).unwrap();
        println!("({}) * {} times: finished in {}.{}s ({}.{}ms/try)", $title, $t, elapsed.as_secs(), elapsed.subsec_nanos()/1_000_000, elapsed_per_entry.subsec_nanos()/1_000_000, elapsed_per_entry.subsec_nanos()/1_000%1_000)
    })
}

fn main() {
    let args = &env::args().collect::<Vec<String>>();
    let t = match args.get(1).unwrap_or(&"10000".to_string()).parse::<isize>() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("usage: {} [t]", args[0]);
            process::exit(1)
        }
    };
    let n = 64*1024;
    let p = 50;
    let K = 2*75;
    measure_time!(format!("sort {} largests with heap in {} buckets with {}% filled", K, n, p), t, {
        run_sort_largests_with_heap(n, p, K)
    });
    measure_time!(format!("fill {}% of {} buckets", p, n), t, {
        run_hashmap32_insert(n, p)
    });
}
