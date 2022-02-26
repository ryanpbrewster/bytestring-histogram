use std::collections::BTreeMap;

use crate::Digest;

#[derive(Default, Debug)]
pub struct QDigest {
    root: Node,
    total_weight: u64,
}

impl QDigest {
    pub fn compress(&mut self, k: u64) {
        self.root.compress(self.total_weight / k);
    }
}
impl Digest for QDigest {
    fn insert(&mut self, key: &[u8], weight: u64) {
        self.total_weight += weight;
        self.root.insert(key, weight);
    }
    fn quantile(&self, p: f64) -> Vec<u8> {
        let mut cur = Vec::new();
        let gas = (self.total_weight as f64 * p) as u64;
        self.root.explore(gas, &mut cur);
        cur
    }
}

#[derive(Default, Debug)]
struct Node {
    weight: u64,
    children: BTreeMap<u8, Node>,
}
impl Node {
    fn insert(&mut self, key: &[u8], weight: u64) {
        if key.is_empty() {
            self.weight += weight;
        } else {
            self.children
                .entry(key[0])
                .or_default()
                .insert(&key[1..], weight);
        }
    }
    fn compress(&mut self, threshold: u64) {
        let mut total = self.weight;
        for child in self.children.values_mut() {
            child.compress(threshold);
            total += child.weight;
        }
        if total <= threshold {
            self.children.retain(|_, child| {
                child.weight = 0;
                !child.children.is_empty()
            });
            self.weight = total;
        }
    }

    // returns how much gas is left after exploring
    fn explore(&self, gas: u64, key: &mut Vec<u8>) -> u64 {
        let mut remaining = gas.saturating_sub(self.weight);
        if remaining == 0 {
            return 0;
        }
        for (&b, child) in &self.children {
            key.push(b);
            remaining = child.explore(remaining, key);
            if remaining == 0 {
                return 0;
            }
            key.pop();
        }
        remaining
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroU64;

    use super::QDigest;
    use crate::Digest;

    #[test]
    fn basic_smoke_test() {
        let mut h = QDigest::default();
        h.insert(b"0011", 9);
        h.insert(b"0022", 9);
        h.insert(b"AA11", 1);
        h.insert(b"AA22", 1);
        h.compress(11);
        assert_eq!(h.quantile(0.1), b"0011".to_vec(),);
        assert_eq!(h.quantile(0.5), b"0022".to_vec(),);
    }

    #[test]
    fn insert_below_node() {
        let mut h = QDigest::default();
        h.insert(b"00", 9);
        h.insert(b"0011", 9);
        h.insert(b"AA11", 1);
        h.insert(b"AA22", 1);
        h.compress(11);
        assert_eq!(h.quantile(0.10), b"00".to_vec(),);
        assert_eq!(h.quantile(0.50), b"0011".to_vec(),);
        assert_eq!(h.quantile(0.95), b"AA1".to_vec(),);
        assert_eq!(h.quantile(1.00), b"AA2".to_vec(),);
    }

    #[quickcheck]
    fn compress_does_not_change_total_weight(inputs: Vec<Vec<u8>>, k: NonZeroU64) -> bool {
        let mut h = QDigest::default();
        for key in inputs {
            h.insert(&key, 1);
        }
        let initial = h.total_weight;
        h.compress(k.into());
        h.total_weight == initial
    }
}