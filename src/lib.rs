use std::collections::{BTreeMap, BinaryHeap};

#[derive(Default, Debug)]
pub struct QDigest {
    root: Node,
}

impl QDigest {
    pub fn insert(&mut self, key: &[u8], weight: u64) {
        let mut node = &mut self.root;
        for &b in key {
            node.weight += weight;
            node = node.children.entry(b).or_default();
        }
        node.weight += weight;
    }

    pub fn compress(&mut self, mut gas: usize) {
        let mut q = BinaryHeap::new();
        q.push(&mut self.root);
        while let Some(node) = q.pop() {
            println!("{} gas left @ {:?}\n", gas, node);
            if node.children.len() > gas {
                node.children.clear();
                continue;
            }
            gas -= node.children.len();
            for child in node.children.values_mut() {
                q.push(child);
            }
        }
    }

    pub fn boundaries(&self) -> Vec<Vec<u8>> {
        let mut acc = Vec::new();
        let mut cur = Vec::new();
        boundaries_helper(&mut acc, &mut cur, &self.root);
        acc
    }
}

fn boundaries_helper(acc: &mut Vec<Vec<u8>>, cur: &mut Vec<u8>, node: &Node) {
    let mut total = 0;
    for (&b, child) in &node.children {
        total += child.weight;
        cur.push(b);
        boundaries_helper(acc, cur, child);
        cur.pop();
    }
    if total < node.weight {
        acc.push(cur.clone());
    }
}

#[derive(Default, Debug)]
struct Node {
    weight: u64,
    children: BTreeMap<u8, Node>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight
    }
}
impl Eq for Node {}
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Node {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight.cmp(&other.weight)
    }
}

#[cfg(test)]
mod tests {
    use crate::QDigest;

    #[test]
    fn it_works() {
        let mut h = QDigest::default();
        h.insert(b"0011", 9);
        println!("{:?}\n", h);
        h.insert(b"0022", 9);
        println!("{:?}\n", h);
        h.insert(b"AA11", 1);
        println!("{:?}\n", h);
        h.insert(b"AA22", 1);
        println!("{:?}\n", h);
        h.compress(8);
        println!("{:?}\n", h);
        assert_eq!(
            h.boundaries(),
            vec![b"0011".to_vec(), b"0022".to_vec(), b"AA".to_vec()]
        );
    }
}