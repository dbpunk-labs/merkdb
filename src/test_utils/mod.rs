mod temp_merk;

use std::ops::Range;
use std::convert::TryInto;
use byteorder::{BigEndian, WriteBytesExt};
use rand::prelude::*;
use crate::tree::{
    Tree,
    Walker,
    NoopCommit,
    Batch,
    Op,
    PanicSource,
    BatchEntry
};

pub use temp_merk::TempMerk;

pub fn assert_tree_invariants(tree: &Tree) {
    assert!(tree.balance_factor().abs() < 2);

    let maybe_left = tree.link(true);
    if let Some(left) = maybe_left {
        assert!(left.key() < tree.key());
        assert!(!left.is_modified());
    }

    let maybe_right = tree.link(false);
    if let Some(right) = maybe_right {
        assert!(right.key() > tree.key());
        assert!(!right.is_modified());
    }

    if let Some(left) = tree.child(true) {
        assert_tree_invariants(left);
    }
    if let Some(right) = tree.child(false) {
        assert_tree_invariants(right);
    }
}

pub fn apply_memonly_unchecked(tree: Tree, batch: &Batch) -> Tree {
    let walker = Walker::<PanicSource>::new(tree, PanicSource {});
    let mut tree = Walker::<PanicSource>::apply_to(Some(walker), batch)
        .expect("apply failed")
        .expect("expected tree");
    tree.commit(&mut NoopCommit {})
        .expect("commit failed");
    tree
}

pub fn apply_memonly(tree: Tree, batch: &Batch) -> Tree {
    let tree = apply_memonly_unchecked(tree, batch);
    assert_tree_invariants(&tree);
    tree
}

pub fn apply_to_memonly(maybe_tree: Option<Tree>, batch: &Batch) -> Option<Tree> {
    match maybe_tree {
        Some(tree) => Some(apply_memonly(tree, batch)),
        None => {
            Walker::<PanicSource>::apply_to(None, batch)
                .expect("apply failed")
                .map(|mut tree| {
                    tree.commit(&mut NoopCommit {}).expect("commit failed");
                    assert_tree_invariants(&tree);
                    tree
                })
        }
    }
}

pub fn put_entry(n: u64) -> BatchEntry {
    let mut key = vec![0; 0];
    key.write_u64::<BigEndian>(n)
        .expect("writing to key failed");
    (key, Op::Put(vec![123; 60]))
}

pub fn del_entry(n: u64) -> BatchEntry {
    let mut key = vec![0; 0];
    key.write_u64::<BigEndian>(n)
        .expect("writing to key failed");
    (key, Op::Delete)
}

pub fn make_batch_seq(range: Range<u64>) -> Vec<BatchEntry> {
    let mut batch = Vec::with_capacity(
        (range.end - range.start).try_into().unwrap()
    );
    for n in range {
        batch.push(put_entry(n));
    }
    batch
}

pub fn make_del_batch_seq(range: Range<u64>) -> Vec<BatchEntry> {
    let mut batch = Vec::with_capacity(
        (range.end - range.start).try_into().unwrap()
    );
    for n in range {
        batch.push(del_entry(n));
    }
    batch
}

pub fn make_batch_rand(size: u64, seed: u64) -> Vec<BatchEntry> {
    let mut rng: SmallRng = SeedableRng::seed_from_u64(seed);
    let mut batch = Vec::with_capacity(size.try_into().unwrap());
    for _ in 0..size {
        let n = rng.gen::<u64>();
        batch.push(put_entry(n));
    }
    batch.sort_by(|a, b| a.0.cmp(&b.0));
    batch
}

pub fn make_del_batch_rand(size: u64, seed: u64) -> Vec<BatchEntry> {
    let mut rng: SmallRng = SeedableRng::seed_from_u64(seed);
    let mut batch = Vec::with_capacity(size.try_into().unwrap());
    for _ in 0..size {
        let n = rng.gen::<u64>();
        batch.push(del_entry(n));
    }
    batch.sort_by(|a, b| a.0.cmp(&b.0));
    batch
}

pub fn random_value(size: usize) -> Vec<u8> {
    let mut value = Vec::with_capacity(size);
    let mut rng = thread_rng();
    rng.fill_bytes(&mut value[..]);
    value
}

pub fn make_mixed_batch_rand(maybe_tree: Option<&Tree>, size: u64) -> Vec<BatchEntry> {
    let mut batch = Vec::with_capacity(size.try_into().unwrap());

    let get_random_key = || {
        let mut rng = thread_rng();
        let tree = maybe_tree.as_ref().unwrap();
        let entries: Vec<_> = tree.iter().collect();
        let index = rng.gen::<u64>() as usize % entries.len();
        entries[index].0.clone()
    };

    let insert = || {
        (random_value(2), Op::Put(random_value(2)))
    };
    let update = || {
        let key = get_random_key();
        (key.to_vec(), Op::Put(random_value(2)))
    };
    let delete = || {
        let key = get_random_key();
        (key.to_vec(), Op::Delete)
    };

    let mut rng = thread_rng();
    for _ in 0..size {
        let entry = if maybe_tree.is_some() {
            let kind = rng.gen::<u64>() % 3;
            if kind == 0 { insert() }
            else if kind == 1 { update() }
            else { delete() }
        } else {
            insert()
        };
        batch.push(entry);
    }
    batch.sort_by(|a, b| a.0.cmp(&b.0));
    batch
}

pub fn make_tree_rand(
    node_count: u64,
    batch_size: u64,
    initial_seed: u64
) -> Tree {
    assert!(node_count >= batch_size);
    assert!((node_count % batch_size) == 0);

    let value = vec![123; 60];
    let mut tree = Tree::new(vec![0; 20], value.clone());

    let mut seed = initial_seed;
    
    let batch_count = node_count / batch_size;
    for _ in 0..batch_count {
        let batch = make_batch_rand(batch_size, seed);
        tree = apply_memonly(tree, &batch);
        seed += 1;
    }

    tree
}

pub fn make_tree_seq(node_count: u64) -> Tree {
    let batch_size = if node_count >= 10_000 {
        assert!(node_count % 10_000 == 0);
        10_000
    } else {
        node_count
    };

    let value = vec![123; 60];
    let mut tree = Tree::new(vec![0; 20], value.clone());
    
    let batch_count = node_count / batch_size;
    for i in 0..batch_count {
        let batch = make_batch_seq((i * batch_size)..((i+1) * batch_size));
        tree = apply_memonly(tree, &batch);
    }

    tree
}
