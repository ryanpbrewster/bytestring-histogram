use std::collections::BTreeMap;

#[derive(Default, Debug)]
pub struct QDigest {
    root: Node,
    total_weight: u64,
}

impl QDigest {
    pub fn insert(&mut self, key: &[u8], weight: u64) {
        self.total_weight += weight;
        self.root.insert(key, weight);
    }

    pub fn compress(&mut self, k: u64) {
        self.root.compress(self.total_weight / k);
    }

    pub fn quantile(&self, p: f64) -> Vec<u8> {
        let mut cur = Vec::new();
        let mut gas = (self.total_weight as f64 * p) as u64;
        explore(&self.root, &mut gas, &mut cur);
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
}

fn explore(node: &Node, gas: &mut u64, key: &mut Vec<u8>) {
    for (&b, child) in &node.children {
        key.push(b);
        explore(child, gas, key);
        if *gas == 0 {
            return;
        }
        key.pop();
    }
    *gas = gas.saturating_sub(node.weight);
}

#[cfg(test)]
mod tests {
    use crate::QDigest;

    #[test]
    fn basic_smoke_test() {
        let mut h = QDigest::default();
        h.insert(b"0011", 9);
        println!("{:?}\n", h);
        h.insert(b"0022", 9);
        println!("{:?}\n", h);
        h.insert(b"AA11", 1);
        println!("{:?}\n", h);
        h.insert(b"AA22", 1);
        println!("{:?}\n", h);
        h.compress(3);
        println!("{:?}\n", h);
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
        h.compress(7);
        assert_eq!(h.quantile(0.99), b"AA".to_vec(),);
    }
}
