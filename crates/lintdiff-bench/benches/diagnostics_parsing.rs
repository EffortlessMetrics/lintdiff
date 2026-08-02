//! Benchmarks for diagnostics parsing performance.
//!
//! Measures the performance of `lintdiff_engine::parse_cargo_messages` across
//! various diagnostic sizes and complexities.

use std::io::Cursor;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use lintdiff_engine::parse_cargo_messages;

/// Generate a single diagnostic JSON message.
fn generate_diagnostic_json(
    level: &str,
    code: &str,
    message: &str,
    file: &str,
    line: u32,
) -> String {
    format!(
        r#"{{"reason":"compiler-message","message":{{"level":"{}","message":"{}","code":{{"code":"{}"}},"spans":[{{"file_name":"{}","line_start":{},"line_end":{},"column_start":1,"column_end":10,"is_primary":true}}]}}}}"#,
        level, message, code, file, line, line
    )
}

/// Generate a diagnostic with multiple spans.
fn generate_multi_span_diagnostic(num_spans: usize) -> String {
    let mut spans = String::new();
    for i in 0..num_spans {
        if i > 0 {
            spans.push(',');
        }
        spans.push_str(&format!(
            r#"{{"file_name":"src/file{}.rs","line_start":{},"line_end":{},"column_start":1,"column_end":10,"is_primary":{}}}"#,
            i,
            i * 10 + 1,
            i * 10 + 5,
            i == 0
        ));
    }

    format!(
        r#"{{"reason":"compiler-message","message":{{"level":"warning","message":"multi-span warning","code":{{"code":"multi_span"}},"spans":[{}]}}}}"#,
        spans
    )
}

/// Generate a JSONL stream with multiple diagnostics.
fn generate_diagnostics_stream(num_diagnostics: usize) -> String {
    let mut stream = String::new();

    for i in 0..num_diagnostics {
        if i > 0 {
            stream.push('\n');
        }

        let level = if i % 3 == 0 { "error" } else { "warning" };
        let code = if i % 2 == 0 {
            format!("clippy::lint_{}", i)
        } else {
            format!("E{:04}", i)
        };

        stream.push_str(&generate_diagnostic_json(
            level,
            &code,
            &format!("Diagnostic message number {}", i),
            &format!("src/module{}.rs", i % 10),
            (i % 100) as u32 + 1,
        ));
    }

    stream
}

/// Generate a JSONL stream with mixed message types (including non-compiler-message).
fn generate_mixed_stream(num_lines: usize) -> String {
    let mut stream = String::new();

    for i in 0..num_lines {
        if i > 0 {
            stream.push('\n');
        }

        match i % 5 {
            0 => {
                // compiler-message
                stream.push_str(&generate_diagnostic_json(
                    "warning",
                    "clippy::test",
                    "Test warning",
                    "src/lib.rs",
                    i as u32 + 1,
                ));
            }
            1 => {
                // compiler-artifact
                stream.push_str(&format!(
                    r#"{{"reason":"compiler-artifact","package_id":"test_pkg {}","target":{{"name":"test"}}}}"#,
                    i
                ));
            }
            2 => {
                // build-script-executed
                stream.push_str(&format!(
                    r#"{{"reason":"build-script-executed","package_id":"build_{}"}}"#,
                    i
                ));
            }
            _ => {
                // compiler-message with error
                stream.push_str(&generate_diagnostic_json(
                    "error",
                    "E0001",
                    &format!("Error number {}", i),
                    "src/main.rs",
                    i as u32 + 1,
                ));
            }
        }
    }

    stream
}

fn bench_diagnostics_parsing(c: &mut Criterion) {
    // Benchmark parsing single diagnostics
    let mut single_group = c.benchmark_group("single_diagnostic");

    let simple_diag = generate_diagnostic_json(
        "warning",
        "clippy::unwrap_used",
        "used unwrap which may panic",
        "src/lib.rs",
        42,
    );

    single_group.bench_function("simple", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&simple_diag));
            parse_cargo_messages(cursor)
        });
    });

    let multi_span = generate_multi_span_diagnostic(5);
    single_group.bench_function("5_spans", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&multi_span));
            parse_cargo_messages(cursor)
        });
    });

    let many_spans = generate_multi_span_diagnostic(20);
    single_group.bench_function("20_spans", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&many_spans));
            parse_cargo_messages(cursor)
        });
    });

    single_group.finish();

    // Benchmark parsing streams of diagnostics
    let mut stream_group = c.benchmark_group("diagnostic_streams");

    for count in [10, 50, 100, 500].iter() {
        let stream = generate_diagnostics_stream(*count);
        stream_group.bench_with_input(
            BenchmarkId::new("pure_diagnostics", count),
            &stream,
            |b, stream| {
                b.iter(|| {
                    let cursor = Cursor::new(black_box(stream));
                    parse_cargo_messages(cursor)
                });
            },
        );
    }

    stream_group.finish();

    // Benchmark parsing mixed streams (more realistic)
    let mut mixed_group = c.benchmark_group("mixed_streams");

    for count in [50, 200, 1000].iter() {
        let stream = generate_mixed_stream(*count);
        mixed_group.bench_with_input(
            BenchmarkId::new("mixed_messages", count),
            &stream,
            |b, stream| {
                b.iter(|| {
                    let cursor = Cursor::new(black_box(stream));
                    parse_cargo_messages(cursor)
                });
            },
        );
    }

    mixed_group.finish();

    // Benchmark large realistic output
    let mut large_group = c.benchmark_group("large_output");

    let large_stream = generate_diagnostics_stream(2000);
    large_group.bench_function("2000_diagnostics", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&large_stream));
            parse_cargo_messages(cursor)
        });
    });

    let large_mixed = generate_mixed_stream(5000);
    large_group.bench_function("5000_mixed", |b| {
        b.iter(|| {
            let cursor = Cursor::new(black_box(&large_mixed));
            parse_cargo_messages(cursor)
        });
    });

    large_group.finish();
}

criterion_group!(benches, bench_diagnostics_parsing);
criterion_main!(benches);
