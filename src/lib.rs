#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

pub trait Digest {
    fn insert(&mut self, key: &[u8], weight: u64);
    fn quantile(&self, p: f64) -> Vec<u8>;
}

pub mod qdigest;

#[cfg(test)]
mod oracle;
