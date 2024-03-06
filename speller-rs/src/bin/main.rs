use speller_rs::Speller;

fn main() {
    println!("language: {:?}", Speller::languages());
    let speller = Speller::builder()
        .build().unwrap();
    let words = ["yessss", "conticorrantue", "obrigada", "oi_biagomes", "haa"];
    for word in words.iter() {
        let start_time = std::time::Instant::now();
        let correct = speller.correction(word);
        println!("The correct word for '{}' is {:?}", word, correct);
        let elapsed = start_time.elapsed();
        println!("Elapsed time: {:?}", elapsed);
    }
}