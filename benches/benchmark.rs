//! Performance benchmarks for flag-rs
//!
//! This benchmark suite compares flag-rs performance characteristics
//! against theoretical alternatives and measures key operations.

use flag_rs::{CommandBuilder, CompletionResult, Flag, FlagType, FlagValue};
use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};

/// Simple benchmark harness
struct Benchmark {
    name: String,
    iterations: usize,
}

impl Benchmark {
    fn new(name: &str, iterations: usize) -> Self {
        Self {
            name: name.to_string(),
            iterations,
        }
    }

    fn run<F>(&self, mut f: F) -> Duration
    where
        F: FnMut(),
    {
        // Warmup
        for _ in 0..10 {
            f();
        }

        // Actual measurement
        let start = Instant::now();
        for _ in 0..self.iterations {
            f();
        }
        start.elapsed()
    }

    fn report(&self, duration: Duration) {
        #[allow(clippy::cast_precision_loss)]
        let per_iter = duration.as_nanos() as f64 / self.iterations as f64;
        println!(
            "{:<40} {:>10.0} ns/iter ({:>6} iterations)",
            self.name, per_iter, self.iterations
        );
    }
}

/// Creates a simple command structure for benchmarking
fn create_simple_cli() -> flag_rs::Command {
    CommandBuilder::new("bench")
        .short("Benchmark CLI")
        .flag(
            Flag::new("verbose")
                .short('v')
                .value_type(FlagType::Bool)
                .default(FlagValue::Bool(false)),
        )
        .flag(Flag::new("output").short('o').value_type(FlagType::String))
        .build()
}

/// Creates a complex nested command structure
fn create_complex_cli() -> flag_rs::Command {
    let mut root = CommandBuilder::new("complex")
        .short("Complex CLI with many subcommands")
        .flag(
            Flag::new("config")
                .short('c')
                .value_type(FlagType::File)
                .default(FlagValue::String("config.yaml".to_string())),
        )
        .build();

    // Add 50 subcommands
    for i in 0..50 {
        let mut sub = CommandBuilder::new(format!("sub{i:02}"))
            .short(format!("Subcommand {i}"))
            .flag(Flag::new("flag1").short('1').value_type(FlagType::String))
            .flag(Flag::new("flag2").short('2').value_type(FlagType::Int))
            .build();

        // Add 10 nested subcommands
        for j in 0..10 {
            sub.add_command(
                CommandBuilder::new(format!("nested{j}"))
                    .short(format!("Nested command {j}"))
                    .flag(Flag::new("deep").value_type(FlagType::Bool))
                    .build(),
            );
        }

        root.add_command(sub);
    }

    root
}

fn bench_command_creation() {
    println!("\n=== Command Creation Benchmarks ===");

    let bench = Benchmark::new("Simple command creation", 10_000);
    let duration = bench.run(|| {
        let _ = create_simple_cli();
    });
    bench.report(duration);

    let bench = Benchmark::new("Complex command creation (50 subs)", 100);
    let duration = bench.run(|| {
        let _ = create_complex_cli();
    });
    bench.report(duration);

    let bench = Benchmark::new("CommandBuilder with 10 flags", 1_000);
    let duration = bench.run(|| {
        let mut cmd = CommandBuilder::new("test");
        for i in 0..10 {
            cmd = cmd.flag(Flag::new(format!("flag{i}")).value_type(FlagType::String));
        }
        let _ = cmd.build();
    });
    bench.report(duration);
}

fn bench_flag_parsing() {
    println!("\n=== Flag Parsing Benchmarks ===");

    let cli = create_simple_cli();

    let bench = Benchmark::new("Parse no flags", 10_000);
    let duration = bench.run(|| {
        let args = vec!["bench".to_string()];
        let _ = cli.execute(args);
    });
    bench.report(duration);

    let bench = Benchmark::new("Parse single flag", 10_000);
    let duration = bench.run(|| {
        let args = vec!["bench".to_string(), "--verbose".to_string()];
        let _ = cli.execute(args);
    });
    bench.report(duration);

    let bench = Benchmark::new("Parse flag with value", 10_000);
    let duration = bench.run(|| {
        let args = vec![
            "bench".to_string(),
            "--output".to_string(),
            "file.txt".to_string(),
        ];
        let _ = cli.execute(args);
    });
    bench.report(duration);

    let bench = Benchmark::new("Parse multiple flags", 10_000);
    let duration = bench.run(|| {
        let args = vec![
            "bench".to_string(),
            "-v".to_string(),
            "-o".to_string(),
            "output.txt".to_string(),
        ];
        let _ = cli.execute(args);
    });
    bench.report(duration);
}

fn bench_subcommand_lookup() {
    println!("\n=== Subcommand Lookup Benchmarks ===");

    let cli = create_complex_cli();

    let bench = Benchmark::new("Find immediate subcommand", 10_000);
    let duration = bench.run(|| {
        let _ = cli.find_subcommand("sub25");
    });
    bench.report(duration);

    let bench = Benchmark::new("Find nested subcommand", 10_000);
    let duration = bench.run(|| {
        if let Some(sub) = cli.find_subcommand("sub25") {
            let _ = sub.find_subcommand("nested5");
        }
    });
    bench.report(duration);

    let bench = Benchmark::new("Execute nested command", 1_000);
    let duration = bench.run(|| {
        let args = vec![
            "complex".to_string(),
            "sub25".to_string(),
            "nested5".to_string(),
            "--deep".to_string(),
        ];
        let _ = cli.execute(args);
    });
    bench.report(duration);
}

fn bench_completion() {
    println!("\n=== Completion Benchmarks ===");

    let cli = CommandBuilder::new("comp")
        .arg_completion(|_ctx, prefix| {
            let items: Vec<String> = (0..100)
                .map(|i| format!("item{i:03}"))
                .filter(|item| item.starts_with(prefix))
                .collect();
            Ok(CompletionResult::new().extend(items))
        })
        .build();

    let ctx = flag_rs::Context::new(vec!["comp".to_string()]);

    let bench = Benchmark::new("Complete with empty prefix (100 items)", 1_000);
    let duration = bench.run(|| {
        let _ = cli.get_completions(&ctx, "", None);
    });
    bench.report(duration);

    let bench = Benchmark::new("Complete with prefix (filtered)", 1_000);
    let duration = bench.run(|| {
        let _ = cli.get_completions(&ctx, "item05", None);
    });
    bench.report(duration);

    // Test completion with descriptions
    let cli_desc = CommandBuilder::new("comp_desc")
        .arg_completion(|_ctx, prefix| {
            let mut result = CompletionResult::new();
            for i in 0..50 {
                let item = format!("option{i:02}");
                if item.starts_with(prefix) {
                    result =
                        result.add_with_description(item, format!("Description for option {i}"));
                }
            }
            Ok(result)
        })
        .build();

    let bench = Benchmark::new("Complete with descriptions (50 items)", 1_000);
    let duration = bench.run(|| {
        let _ = cli_desc.get_completions(&ctx, "", None);
    });
    bench.report(duration);
}

fn bench_flag_validation() {
    println!("\n=== Flag Validation Benchmarks ===");

    let cli = CommandBuilder::new("validate")
        .flag(Flag::new("choice").value_type(FlagType::Choice(vec![
            "opt1".to_string(),
            "opt2".to_string(),
            "opt3".to_string(),
        ])))
        .flag(Flag::new("range").value_type(FlagType::Range(1, 100)))
        .flag(
            Flag::new("required")
                .value_type(FlagType::String)
                .required(),
        )
        .build();

    let bench = Benchmark::new("Validate choice flag", 10_000);
    let duration = bench.run(|| {
        let args = vec![
            "validate".to_string(),
            "--choice".to_string(),
            "opt2".to_string(),
            "--required".to_string(),
            "value".to_string(),
        ];
        let _ = cli.execute(args);
    });
    bench.report(duration);

    let bench = Benchmark::new("Validate range flag", 10_000);
    let duration = bench.run(|| {
        let args = vec![
            "validate".to_string(),
            "--range".to_string(),
            "50".to_string(),
            "--required".to_string(),
            "value".to_string(),
        ];
        let _ = cli.execute(args);
    });
    bench.report(duration);

    let bench = Benchmark::new("Validate missing required flag", 10_000);
    let duration = bench.run(|| {
        let args = vec!["validate".to_string()];
        let _ = cli.execute(args); // This will fail validation
    });
    bench.report(duration);
}

fn bench_memory_optimizations() {
    println!("\n=== Memory Optimization Benchmarks ===");

    // Test string interning
    let bench = Benchmark::new("String interning (100 strings)", 1_000);
    let duration = bench.run(|| {
        use flag_rs::string_pool;
        for i in 0..100 {
            let _ = string_pool::intern(&format!("flag{i}"));
        }
    });
    bench.report(duration);

    // Test optimized completion
    let bench = Benchmark::new("Optimized completion result", 1_000);
    let duration = bench.run(|| {
        use flag_rs::completion_optimized::CompletionResultOptimized;
        use std::borrow::Cow;

        let mut result = CompletionResultOptimized::new();
        for i in 0..50 {
            result = result.add_with_description(
                Cow::Owned(format!("item{i}")),
                Cow::Borrowed("Static description"),
            );
        }
        let _ = result.into_legacy();
    });
    bench.report(duration);
}

fn bench_help_generation() {
    println!("\n=== Help Generation Benchmarks ===");

    let simple = create_simple_cli();
    let complex = create_complex_cli();

    let bench = Benchmark::new("Generate help for simple command", 1_000);
    let duration = bench.run(|| {
        // Capture help output to avoid printing
        let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
            simple.print_help();
        }));
    });
    bench.report(duration);

    let bench = Benchmark::new("Generate help for complex command", 100);
    let duration = bench.run(|| {
        // Capture help output to avoid printing
        let _ = std::panic::catch_unwind(AssertUnwindSafe(|| {
            complex.print_help();
        }));
    });
    bench.report(duration);
}

fn main() {
    println!("=== Flag-rs Performance Benchmarks ===");
    println!("Running benchmarks... (this may take a minute)\n");

    let total_start = Instant::now();

    bench_command_creation();
    bench_flag_parsing();
    bench_subcommand_lookup();
    bench_completion();
    bench_flag_validation();
    bench_memory_optimizations();
    bench_help_generation();

    let total_duration = total_start.elapsed();
    println!("\n=== Summary ===");
    println!("Total benchmark time: {:.2?}", total_duration);

    println!("\n=== Performance Characteristics ===");
    println!("• Command creation: O(n) where n = number of flags/subcommands");
    println!("• Flag parsing: O(n) where n = number of arguments");
    println!("• Subcommand lookup: O(1) average (HashMap)");
    println!("• Completion: O(n*m) where n = items, m = prefix length");
    println!("• Memory usage: Optimized with string interning and Cow strings");

    println!("\n=== Comparison Notes ===");
    println!("While we don't have direct clap benchmarks (zero dependencies),");
    println!("flag-rs focuses on:");
    println!("• Dynamic completions (unique feature)");
    println!("• Memory efficiency for large CLIs");
    println!("• Fast subcommand lookup with HashMaps");
    println!("• Minimal allocations during parsing");
}
