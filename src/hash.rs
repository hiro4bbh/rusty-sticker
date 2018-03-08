use std::hash;

pub struct Hasher(u64);

impl Default for Hasher {
    fn default() -> Hasher {
        Hasher(0)
    }
}

impl hash::Hasher for Hasher {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, data: &[u8]) {
        let Hasher(mut h) = *self;
        for x in data {
            h = h*31 + (*x as u64);
        }
        *self = Hasher(h);
    }
}
