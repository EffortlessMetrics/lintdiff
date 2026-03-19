//! Benchmarks for diff parsing performance.
//!
//! Measures the performance of `lintdiff_diff::parse_unified_diff` across
//! various diff sizes and complexities.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

use lintdiff_diff::parse_unified_diff;

/// Generate a simple diff with a single file and specified number of hunks/lines.
fn generate_simple_diff(num_hunks: usize, lines_per_hunk: usize) -> String {
    let mut diff = String::new();

    diff.push_str("diff --git a/src/lib.rs b/src/lib.rs\n");
    diff.push_str("--- a/src/lib.rs\n");
    diff.push_str("+++ b/src/lib.rs\n");

    for hunk in 0..num_hunks {
        let old_start = 1 + (hunk * lines_per_hunk);
        let new_start = 1 + (hunk * lines_per_hunk);
        diff.push_str(&format!(
            "@@ -{},{} +{},{} @@\n",
            old_start,
            lines_per_hunk,
            new_start,
            lines_per_hunk + 1
        ));

        for i in 0..lines_per_hunk {
            diff.push_str(&format!("+// Added line {} in hunk {}\n", i, hunk));
        }
    }

    diff
}

/// Generate a multi-file diff with specified number of files.
fn generate_multi_file_diff(
    num_files: usize,
    hunks_per_file: usize,
    lines_per_hunk: usize,
) -> String {
    let mut diff = String::new();

    for file in 0..num_files {
        diff.push_str(&format!(
            "diff --git a/src/file{}.rs b/src/file{}.rs\n",
            file, file
        ));
        diff.push_str(&format!("--- a/src/file{}.rs\n", file));
        diff.push_str(&format!("+++ b/src/file{}.rs\n", file));

        for hunk in 0..hunks_per_file {
            let old_start = 1 + (hunk * lines_per_hunk);
            let new_start = 1 + (hunk * lines_per_hunk);
            diff.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                old_start,
                lines_per_hunk,
                new_start,
                lines_per_hunk + 1
            ));

            for i in 0..lines_per_hunk {
                diff.push_str(&format!("+// File {} hunk {} line {}\n", file, hunk, i));
            }
        }
    }

    diff
}

/// Generate a diff with renames.
fn generate_rename_diff(num_renames: usize) -> String {
    let mut diff = String::new();

    for i in 0..num_renames {
        diff.push_str(&format!("diff --git a/src/old{}.rs b/src/new{}.rs\n", i, i));
        diff.push_str(&format!("rename from src/old{}.rs\n", i));
        diff.push_str(&format!("rename to src/new{}.rs\n", i));
        diff.push_str("--- a/src/old{}.rs\n");
        diff.push_str("+++ b/src/new{}.rs\n");
        diff.push_str("@@ -1,3 +1,3 @@\n");
        diff.push_str(" // Renamed file\n");
        diff.push_str("-// Old content\n");
        diff.push_str("+// New content\n");
    }

    diff
}

fn bench_diff_parsing(c: &mut Criterion) {
    // Benchmark parsing small diffs (typical PR size)
    let mut small_group = c.benchmark_group("small_diffs");

    let tiny_diff = "diff --git a/src/lib.rs b/src/lib.rs
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,0 +1,1 @@
+fn main() {}
";
    small_group.bench_function("tiny", |b| {
        b.iter(|| parse_unified_diff(black_box(tiny_diff)));
    });

    let small_diff = generate_simple_diff(3, 10);
    small_group.bench_function("3_hunks_10_lines", |b| {
        b.iter(|| parse_unified_diff(black_box(&small_diff)));
    });

    small_group.finish();

    // Benchmark parsing medium diffs
    let mut medium_group = c.benchmark_group("medium_diffs");

    for lines_per_hunk in [10, 50, 100].iter() {
        let diff = generate_simple_diff(10, *lines_per_hunk);
        medium_group.bench_with_input(
            BenchmarkId::new("10_hunks", lines_per_hunk),
            &diff,
            |b, diff| {
                b.iter(|| parse_unified_diff(black_box(diff)));
            },
        );
    }

    medium_group.finish();

    // Benchmark parsing large diffs
    let mut large_group = c.benchmark_group("large_diffs");

    let large_diff = generate_simple_diff(50, 100);
    large_group.bench_function("50_hunks_100_lines", |b| {
        b.iter(|| parse_unified_diff(black_box(&large_diff)));
    });

    let very_large_diff = generate_simple_diff(100, 200);
    large_group.bench_function("100_hunks_200_lines", |b| {
        b.iter(|| parse_unified_diff(black_box(&very_large_diff)));
    });

    large_group.finish();

    // Benchmark multi-file diffs
    let mut multi_group = c.benchmark_group("multi_file_diffs");

    for num_files in [5, 20, 50].iter() {
        let diff = generate_multi_file_diff(*num_files, 3, 10);
        multi_group.bench_with_input(BenchmarkId::new("files", num_files), &diff, |b, diff| {
            b.iter(|| parse_unified_diff(black_box(diff)));
        });
    }

    multi_group.finish();

    // Benchmark rename detection
    let mut rename_group = c.benchmark_group("rename_diffs");

    let rename_diff = generate_rename_diff(20);
    rename_group.bench_function("20_renames", |b| {
        b.iter(|| parse_unified_diff(black_box(&rename_diff)));
    });

    rename_group.finish();
}

criterion_group!(benches, bench_diff_parsing);
criterion_main!(benches);
