use std::{collections::BTreeMap, num::NonZeroU64};

use crate::{Digest, qdigest::QDigest};

#[derive(Default)]
struct Oracle {
    total_weight: u64,
    data: BTreeMap<Vec<u8>, u64>,
}
impl Digest for Oracle {
    fn insert(&mut self, key: &[u8], weight: u64) {
        self.total_weight += weight;
        *self.data.entry(key.to_vec()).or_default() += weight;
    }

    fn quantile(&self, p: f64) -> Vec<u8> {
        let mut gas = (self.total_weight as f64 * p) as u64;
        for (key, &weight) in &self.data {
            if gas <= weight {
                return key.clone();
            }
            gas -= weight;
        }
        vec![]
    }
}

#[quickcheck]
fn qc_qdigest_generates_lower_bounds(inputs: Vec<Vec<u8>>, k: NonZeroU64, r: u32) -> bool {
    let mut oracle = Oracle::default();
    let mut h = QDigest::default();

    for xs in inputs {
        oracle.insert(&xs, 1);
        h.insert(&xs, 1);
    }
    h.compress(k.into());
    let p = r as f64 / u32::MAX as f64;
    h.quantile(p) <= oracle.quantile(p)
}
