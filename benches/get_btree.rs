#[macro_use]
extern crate criterion;

use criterion::Criterion;

use std::collections::BTreeMap;

fn get(map: &BTreeMap<u32, u32>) {
    for i in 0..10_000 {
        assert!(map.get(&i) == Some(&i));
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut map = BTreeMap::new();
    for i in 0..10_000 {
        let _ = map.insert(i, i);
    }
    c.bench_function("get_btree", move |b| b.iter(|| get(&map)));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
