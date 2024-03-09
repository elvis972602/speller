use criterion::{criterion_group, criterion_main, Criterion};
use levenshtein_automata::{Distance, LevenshteinAutomatonBuilder};
use std::{cmp, mem};

pub fn edit_distance(a: &str, b: &str, limit: usize) -> Option<usize> {
    let mut a = &a.chars().collect::<Vec<_>>()[..];
    let mut b = &b.chars().collect::<Vec<_>>()[..];

    // Ensure that `b` is the shorter string, minimizing memory use.
    if a.len() < b.len() {
        mem::swap(&mut a, &mut b);
    }

    let min_dist = a.len() - b.len();
    // If we know the limit will be exceeded, we can return early.
    if min_dist > limit {
        return None;
    }

    // Strip common prefix.
    while let Some((b_char, b_rest)) = b.split_first() {
        if let Some((a_char, a_rest)) = a.split_first() {
            if a_char == b_char {
                a = a_rest;
                b = b_rest;
                continue;
            }
        }
        break;
    }
    // Strip common suffix.
    while let Some((b_char, b_rest)) = b.split_last() {
        if let Some((a_char, a_rest)) = a.split_last() {
            if a_char == b_char {
                a = a_rest;
                b = b_rest;
                continue;
            }
        }
        break;
    }

    // If either string is empty, the distance is the length of the other.
    // We know that `b` is the shorter string, so we don't need to check `a`.
    if b.len() == 0 {
        return Some(min_dist);
    }

    let mut prev_prev = vec![usize::MAX; b.len() + 1];
    let mut prev = (0..=b.len()).collect::<Vec<_>>();
    let mut current = vec![0; b.len() + 1];

    // row by row
    for i in 1..=a.len() {
        current[0] = i;
        let a_idx = i - 1;

        // column by column
        for j in 1..=b.len() {
            let b_idx = j - 1;

            // There is no cost to substitute a character with itself.
            let substitution_cost = if a[a_idx] == b[b_idx] { 0 } else { 1 };

            current[j] = cmp::min(
                // deletion
                prev[j] + 1,
                cmp::min(
                    // insertion
                    current[j - 1] + 1,
                    // substitution
                    prev[j - 1] + substitution_cost,
                ),
            );

            if (i > 1) && (j > 1) && (a[a_idx] == b[b_idx - 1]) && (a[a_idx - 1] == b[b_idx]) {
                // transposition
                current[j] = cmp::min(current[j], prev_prev[j - 2] + 1);
            }
        }

        // Rotate the buffers, reusing the memory.
        [prev_prev, prev, current] = [prev, current, prev_prev];
    }

    // `prev` because we already rotated the buffers.
    let distance = prev[b.len()];
    (distance <= limit).then_some(distance)
}

pub fn edit_distance_automaton(
    automaton_builder: LevenshteinAutomatonBuilder,
    word1: &str,
    word2: &str,
) -> Option<u8> {
    let dfa = automaton_builder.build_dfa(&word1);
    match dfa.eval(word2) {
        Distance::Exact(distance) => Some(distance),
        _ => None,
    }
}

fn levenshtein_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Levenshtein");
    let word1 = "kitten";
    let word2 = "sitting";
    let limit = 3;
    group.bench_function("edit_distance", |b| {
        b.iter(|| edit_distance(word1, word2, limit))
    });
    group.bench_function("edit_distance_automaton", |b| {
        b.iter(|| {
            edit_distance_automaton(
                LevenshteinAutomatonBuilder::new(limit as u8, true),
                word1,
                word2,
            )
        })
    });
    group.finish();
}

criterion_group!(benches, levenshtein_benchmark);
criterion_main!(benches);
