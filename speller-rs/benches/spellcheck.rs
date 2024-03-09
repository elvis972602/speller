use criterion::{criterion_group, criterion_main, Criterion};
use speller_rs::Speller;

fn spellcheck() {
    let speller = Speller::builder()
        .dict_file(vec!["../data/en.json".to_string()])
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
    config = Criterion::default().sample_size(10);
    targets = criterion_benchmark
);

criterion_main!(benches);
