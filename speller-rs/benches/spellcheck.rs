use criterion::{Criterion, criterion_group, criterion_main};
use speller::Speller;

fn spellcheck() {
    let speller = Speller::builder().build().unwrap();
    let word = "Melanynijholtxo";
    // let start_time = std::time::Instant::now();
    let correct = speller.correct(word);
    // let elapsed = start_time.elapsed();
    // println!("Elapsed time: {:?}", elapsed);
    // println!("The correct word for '{}' is {:?}", word, correct);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("spellcheck", |b| b.iter(|| spellcheck()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
