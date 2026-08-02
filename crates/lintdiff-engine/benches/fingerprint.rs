//! Benchmarks for fingerprint generation performance.
//!
//! Measures the performance of `lintdiff_engine::fingerprint` across
//! various input sizes and configurations.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

use lintdiff_engine::fingerprint;
use lintdiff_types::{Location, NormPath};

/// Generate a message of specified length with realistic content.
fn generate_message(length: usize) -> String {
    let base = "variable `x` is unused in function `process_data` at module::handler::process ";
    let repeated = base.repeat(length / base.len() + 1);
    repeated[..length].to_string()
}

/// Generate a path with specified depth.
fn generate_path(depth: usize) -> String {
    let components: Vec<String> = (0..depth).map(|i| format!("level{}", i)).collect();
    let mut result = components.join("/");
    result.push_str("/mod.rs");
    result
}

fn bench_fingerprint(c: &mut Criterion) {
    // Benchmark basic fingerprint generation
    let mut basic_group = c.benchmark_group("basic_fingerprint");

    basic_group.bench_function("no_location", |b| {
        b.iter(|| {
            fingerprint(
                black_box("clippy::unwrap_used"),
                black_box(None),
                black_box("called `.unwrap()` on an `Option` value"),
            )
        });
    });

    let loc = Location {
        path: NormPath::new("src/lib.rs"),
        line: Some(42),
        col: Some(10),
    };
    basic_group.bench_function("with_location", |b| {
        b.iter(|| {
            fingerprint(
                black_box("clippy::unwrap_used"),
                black_box(Some(&loc)),
                black_box("called `.unwrap()` on an `Option` value"),
            )
        });
    });

    basic_group.finish();

    // Benchmark with varying message lengths
    let mut message_group = c.benchmark_group("message_length");

    for length in [50, 200, 500, 1000, 5000].iter() {
        let message = generate_message(*length);
        message_group.bench_with_input(BenchmarkId::new("msg_len", length), &message, |b, msg| {
            b.iter(|| fingerprint(black_box("E0001"), black_box(None), black_box(msg.as_str())));
        });
    }

    message_group.finish();

    // Benchmark with varying path depths
    let mut path_group = c.benchmark_group("path_depth");

    for depth in [1, 3, 5, 10, 20].iter() {
        let path = generate_path(*depth);
        let loc = Location {
            path: NormPath::new(&path),
            line: Some(100),
            col: Some(5),
        };
        path_group.bench_with_input(BenchmarkId::new("depth", depth), &loc, |b, loc| {
            b.iter(|| {
                fingerprint(
                    black_box("clippy::complexity"),
                    black_box(Some(loc)),
                    black_box("complex type definition"),
                )
            });
        });
    }

    path_group.finish();

    // Benchmark with varying code lengths
    let mut code_group = c.benchmark_group("code_length");

    let codes = [
        "E001",
        "clippy::unwrap_used",
        "rustc::type_length_limit",
        "clippy::large_types_passed_by_value",
        "very_long_diagnostic_code_name_that_exceeds_normal_length",
    ];

    for code in codes.iter() {
        code_group.bench_with_input(BenchmarkId::new("code", code.len()), code, |b, code| {
            b.iter(|| {
                fingerprint(
                    black_box(*code),
                    black_box(None),
                    black_box("sample message"),
                )
            });
        });
    }

    code_group.finish();

    // Benchmark batch fingerprint generation (simulating realistic workload)
    let mut batch_group = c.benchmark_group("batch_generation");

    let batch_data: Vec<(String, Option<Location>, String)> = (0..100)
        .map(|i| {
            let code = if i % 2 == 0 {
                format!("clippy::lint_{}", i)
            } else {
                format!("E{:04}", i)
            };
            let loc = if i % 3 == 0 {
                None
            } else {
                Some(Location {
                    path: NormPath::new(format!("src/module{}.rs", i % 10)),
                    line: Some((i % 100) as u32 + 1),
                    col: Some((i % 80) as u32 + 1),
                })
            };
            let msg = format!(
                "Diagnostic message for item number {} with some additional context",
                i
            );
            (code, loc, msg)
        })
        .collect();

    batch_group.bench_function("100_fingerprints", |b| {
        b.iter(|| {
            for (code, loc, msg) in &batch_data {
                black_box(fingerprint(code, loc.as_ref(), msg));
            }
        });
    });

    batch_group.finish();

    // Benchmark whitespace normalization
    let mut whitespace_group = c.benchmark_group("whitespace_normalization");

    let clean_msg = "This is a clean message without extra whitespace";
    let messy_msg = "  This   is   a   message   with   lots   of   extra   whitespace  \n\t  ";

    whitespace_group.bench_function("clean_message", |b| {
        b.iter(|| fingerprint(black_box("CODE"), black_box(None), black_box(clean_msg)));
    });

    whitespace_group.bench_function("messy_message", |b| {
        b.iter(|| fingerprint(black_box("CODE"), black_box(None), black_box(messy_msg)));
    });

    whitespace_group.finish();
}

criterion_group!(benches, bench_fingerprint);
criterion_main!(benches);
