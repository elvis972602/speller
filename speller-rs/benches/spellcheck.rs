use criterion::{criterion_group, criterion_main, Criterion};
use speller_rs::Speller;
use std::time::Duration;

fn spellcheck() {
    let speller = Speller::builder(vec!["../data/en.json".to_string()])
        .language(vec![])
        .build()
        .unwrap();
    let word = "zayraizquierdo_";
    let _ = speller.correction(word);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("spellcheck", |b| b.iter(|| spellcheck()));
}

criterion_group!(
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(100));
    targets = criterion_benchmark
);

criterion_main!(benches);
