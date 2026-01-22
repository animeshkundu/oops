//! Performance benchmarks for oops.
//!
//! Compares startup time and rule matching performance between
//! oops (Rust) and the original Python thefuck implementation.
//!
//! Run benchmarks with: `cargo bench`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::process::Command as ProcessCommand;
use std::time::{Duration, Instant};

// Import oops types
use oops::core::{Command as TfCommand, Rule};
use oops::rules::get_all_rules;

/// Benchmark the startup time of the Rust binary.
fn bench_rust_startup(c: &mut Criterion) {
    let mut group = c.benchmark_group("startup");

    // Measure how long it takes to run --version
    group.bench_function("rust_version", |b| {
        b.iter(|| {
            let output = ProcessCommand::new(env!("CARGO_BIN_EXE_oops"))
                .arg("--version")
                .output()
                .expect("Failed to execute binary");
            black_box(output)
        })
    });

    // Measure how long it takes to run --help
    group.bench_function("rust_help", |b| {
        b.iter(|| {
            let output = ProcessCommand::new(env!("CARGO_BIN_EXE_oops"))
                .arg("--help")
                .output()
                .expect("Failed to execute binary");
            black_box(output)
        })
    });

    // Measure how long it takes to generate an alias
    group.bench_function("rust_alias_bash", |b| {
        b.iter(|| {
            let output = ProcessCommand::new(env!("CARGO_BIN_EXE_oops"))
                .env("TF_SHELL", "bash")
                .arg("--alias")
                .output()
                .expect("Failed to execute binary");
            black_box(output)
        })
    });

    group.finish();
}

/// Benchmark Python thefuck startup time (if available).
fn bench_python_startup(c: &mut Criterion) {
    // Check if Python thefuck is available
    let python_available = ProcessCommand::new("python")
        .args(["-c", "import thefuck"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !python_available {
        eprintln!("Skipping Python benchmarks: thefuck Python module not available");
        return;
    }

    let mut group = c.benchmark_group("startup_comparison");

    // Python startup with --version
    group.bench_function("python_version", |b| {
        b.iter(|| {
            let output = ProcessCommand::new("python")
                .args(["-m", "thefuck", "--version"])
                .output()
                .expect("Failed to execute Python thefuck");
            black_box(output)
        })
    });

    // Rust startup with --version for comparison
    group.bench_function("rust_version", |b| {
        b.iter(|| {
            let output = ProcessCommand::new(env!("CARGO_BIN_EXE_oops"))
                .arg("--version")
                .output()
                .expect("Failed to execute binary");
            black_box(output)
        })
    });

    group.finish();
}

/// Benchmark rule loading performance.
fn bench_rule_loading(c: &mut Criterion) {
    c.bench_function("load_all_rules", |b| {
        b.iter(|| {
            let rules = get_all_rules();
            black_box(rules.len())
        })
    });
}

/// Benchmark rule matching performance.
fn bench_rule_matching(c: &mut Criterion) {
    let rules = get_all_rules();

    let mut group = c.benchmark_group("rule_matching");

    // Test cases for benchmarking
    let test_cases = vec![
        ("sudo_match", "apt install vim", "Permission denied"),
        ("sudo_no_match", "ls /home", "file1 file2"),
        ("cd_parent_match", "cd..", "command not found"),
        ("cd_mkdir_match", "cd newdir", "no such file or directory"),
        ("no_command_match", "gti status", "gti: command not found"),
        (
            "complex_output",
            "git push origin master",
            "error: failed to push some refs to 'origin'\n\
          hint: Updates were rejected because the remote contains work\n\
          hint: that you do not have locally. This is usually caused by\n\
          hint: another repository pushing to the same ref.",
        ),
    ];

    for (name, script, output) in test_cases {
        let cmd = TfCommand::new(script, output);

        group.bench_with_input(BenchmarkId::new("all_rules", name), &cmd, |b, cmd| {
            b.iter(|| {
                let mut matches = Vec::new();
                for rule in rules.iter() {
                    if rule.is_match(cmd) {
                        matches.push(rule.name());
                    }
                }
                black_box(matches)
            })
        });
    }

    group.finish();
}

/// Benchmark individual rule matching performance.
fn bench_individual_rules(c: &mut Criterion) {
    let rules = get_all_rules();

    let mut group = c.benchmark_group("individual_rules");

    // Find specific rules
    let sudo_rule = rules.iter().find(|r| r.name() == "sudo");
    let cd_parent_rule = rules.iter().find(|r| r.name() == "cd_parent");
    let cd_mkdir_rule = rules.iter().find(|r| r.name() == "cd_mkdir");
    let no_command_rule = rules.iter().find(|r| r.name() == "no_command");

    // Benchmark sudo rule
    if let Some(rule) = sudo_rule {
        let matching_cmd = TfCommand::new("apt install vim", "Permission denied");
        let non_matching_cmd = TfCommand::new("ls /home", "file1 file2");

        group.bench_function("sudo_is_match_true", |b| {
            b.iter(|| black_box(rule.is_match(&matching_cmd)))
        });

        group.bench_function("sudo_is_match_false", |b| {
            b.iter(|| black_box(rule.is_match(&non_matching_cmd)))
        });

        group.bench_function("sudo_get_new_command", |b| {
            b.iter(|| black_box(rule.get_new_command(&matching_cmd)))
        });
    }

    // Benchmark cd_parent rule
    if let Some(rule) = cd_parent_rule {
        let matching_cmd = TfCommand::new("cd..", "command not found");

        group.bench_function("cd_parent_is_match", |b| {
            b.iter(|| black_box(rule.is_match(&matching_cmd)))
        });

        group.bench_function("cd_parent_get_new_command", |b| {
            b.iter(|| black_box(rule.get_new_command(&matching_cmd)))
        });
    }

    // Benchmark cd_mkdir rule
    if let Some(rule) = cd_mkdir_rule {
        let matching_cmd = TfCommand::new("cd newdir", "no such file or directory");

        group.bench_function("cd_mkdir_is_match", |b| {
            b.iter(|| black_box(rule.is_match(&matching_cmd)))
        });

        group.bench_function("cd_mkdir_get_new_command", |b| {
            b.iter(|| black_box(rule.get_new_command(&matching_cmd)))
        });
    }

    // Benchmark no_command rule
    if let Some(rule) = no_command_rule {
        let matching_cmd = TfCommand::new("gti status", "gti: command not found");

        group.bench_function("no_command_is_match", |b| {
            b.iter(|| black_box(rule.is_match(&matching_cmd)))
        });

        // Note: get_new_command for no_command is expensive as it searches PATH
        // We run it with a smaller iteration count
        group.bench_function("no_command_get_new_command", |b| {
            b.iter(|| black_box(rule.get_new_command(&matching_cmd)))
        });
    }

    group.finish();
}

/// Benchmark command parsing performance.
fn bench_command_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("command_parsing");

    let test_scripts = vec![
        ("simple", "git status"),
        ("with_args", "git commit -m 'Initial commit'"),
        (
            "complex",
            "docker run -it --rm -v /home/user:/data ubuntu:latest /bin/bash",
        ),
        (
            "long",
            "find . -name '*.rs' -type f -exec grep -l 'pattern' {} \\;",
        ),
    ];

    for (name, script) in test_scripts {
        group.bench_with_input(
            BenchmarkId::new("script_parts", name),
            &script,
            |b, &script| {
                b.iter(|| {
                    let cmd = TfCommand::new(script, "");
                    black_box(cmd.script_parts())
                })
            },
        );
    }

    group.finish();
}

/// Benchmark full correction workflow.
fn bench_full_correction(c: &mut Criterion) {
    let rules = get_all_rules();

    let mut group = c.benchmark_group("full_correction");

    let test_cases = vec![
        ("sudo_case", "apt install vim", "Permission denied"),
        ("cd_case", "cd..", "command not found"),
        ("no_match_case", "ls /home", "file1 file2"),
    ];

    for (name, script, output) in test_cases {
        let cmd = TfCommand::new(script, output);

        group.bench_with_input(BenchmarkId::new("get_corrections", name), &cmd, |b, cmd| {
            b.iter(|| {
                let mut all_corrections = Vec::new();
                for rule in rules.iter() {
                    if rule.is_match(cmd) {
                        all_corrections.extend(rule.get_new_command(cmd));
                    }
                }
                black_box(all_corrections)
            })
        });
    }

    group.finish();
}

/// Benchmark memory usage by creating many commands.
fn bench_memory_pressure(c: &mut Criterion) {
    c.bench_function("create_1000_commands", |b| {
        b.iter(|| {
            let commands: Vec<TfCommand> = (0..1000)
                .map(|i| {
                    TfCommand::new(
                        format!("command{} arg1 arg2", i),
                        format!("output line 1\noutput line 2\nerror: {}", i),
                    )
                })
                .collect();
            black_box(commands.len())
        })
    });
}

/// Custom startup time measurement (more accurate than criterion for cold starts).
fn measure_startup_times() -> (Duration, Option<Duration>) {
    // Measure Rust startup
    let rust_start = Instant::now();
    let _ = ProcessCommand::new(env!("CARGO_BIN_EXE_oops"))
        .arg("--version")
        .output();
    let rust_duration = rust_start.elapsed();

    // Measure Python startup if available
    let python_duration = if ProcessCommand::new("python")
        .args(["-c", "import thefuck"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        let python_start = Instant::now();
        let _ = ProcessCommand::new("python")
            .args(["-m", "thefuck", "--version"])
            .output();
        Some(python_start.elapsed())
    } else {
        None
    };

    (rust_duration, python_duration)
}

/// Print startup time comparison at the end of benchmarks.
fn print_startup_comparison() {
    println!("\n=== Startup Time Comparison (single run) ===");

    // Run multiple times and average
    let mut rust_times = Vec::new();
    let mut python_times = Vec::new();

    for _ in 0..5 {
        let (rust, python) = measure_startup_times();
        rust_times.push(rust);
        if let Some(p) = python {
            python_times.push(p);
        }
    }

    let rust_avg: Duration = rust_times.iter().sum::<Duration>() / rust_times.len() as u32;
    println!("Rust average startup time: {:?}", rust_avg);

    if !python_times.is_empty() {
        let python_avg: Duration =
            python_times.iter().sum::<Duration>() / python_times.len() as u32;
        println!("Python average startup time: {:?}", python_avg);

        let speedup = python_avg.as_secs_f64() / rust_avg.as_secs_f64();
        println!("Rust is {:.1}x faster than Python", speedup);
    } else {
        println!("Python thefuck not available for comparison");
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(50)
        .measurement_time(Duration::from_secs(5));
    targets =
        bench_rust_startup,
        bench_python_startup,
        bench_rule_loading,
        bench_rule_matching,
        bench_individual_rules,
        bench_command_parsing,
        bench_full_correction,
        bench_memory_pressure
}

criterion_main!(benches);

// Uncomment to run startup comparison manually
// #[test]
// fn test_print_startup_comparison() {
//     print_startup_comparison();
// }
