use std::time::Duration;
use criterion::{Criterion, criterion_group, criterion_main};
use speller_rs::Speller;

fn spellcheck() {
    let speller = Speller::builder().
        language(vec![]).
        local_dictionary(Some(vec!["../data/en.json".to_string()])).
        build().unwrap();
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
