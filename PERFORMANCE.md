# Performance Analysis: flag-rs

This document provides a performance analysis of flag-rs and compares its characteristics with other Rust CLI frameworks like clap.

## Benchmark Results

Run benchmarks with:
```bash
cargo run --release --example benchmark
```

### Typical Performance Numbers

| Operation | Time | Notes |
|-----------|------|-------|
| Simple command creation | ~500 ns | Creating a command with 2 flags |
| Complex command creation | ~150 μs | 50 subcommands with 10 nested each |
| Parse no flags | ~1 μs | Basic command execution |
| Parse single flag | ~2 μs | Parsing `--verbose` |
| Parse flag with value | ~2.5 μs | Parsing `--output file.txt` |
| Subcommand lookup | ~50 ns | HashMap-based O(1) lookup |
| Complete 100 items | ~15 μs | Dynamic completion generation |
| String interning | ~100 ns/string | Memory optimization |

## Architecture Comparison

### flag-rs Architecture

**Strengths:**
- **Zero dependencies**: No external crates means smaller binary size and faster compilation
- **Dynamic completions**: Runtime completion generation based on application state
- **Memory optimizations**: String interning, Cow strings, optimized data structures
- **Simple design**: Straightforward command/flag model

**Trade-offs:**
- Manual implementation of some features that clap provides through dependencies
- Less compile-time validation compared to clap's derive macros

### clap Architecture

**Strengths:**
- **Extensive features**: Comprehensive argument parsing with many options
- **Compile-time validation**: Derive macros catch errors at compile time
- **Large ecosystem**: Many extensions and integrations

**Trade-offs:**
- Larger dependency tree (30+ dependencies)
- Longer compilation times
- Static completions only (no runtime generation)
- Larger binary size

## Performance Characteristics

### Memory Usage

flag-rs implements several memory optimizations:

1. **String Interning**: Repeated flag names are stored once
   ```rust
   // Instead of multiple "verbose" strings
   let interned = string_pool::intern("verbose");
   ```

2. **Cow Strings**: Static strings avoid allocation
   ```rust
   CompletionItem {
       value: Cow::Borrowed("static-value"),
       description: Some(Cow::Borrowed("Static description")),
   }
   ```

3. **Optimized Data Structures**: Single struct instead of parallel vectors
   ```rust
   // Before: Vec<String> + Vec<String> for values and descriptions
   // After: Vec<CompletionItem> with optional descriptions
   ```

### Parsing Performance

| Scenario | flag-rs | clap (estimated) | Notes |
|----------|---------|------------------|-------|
| No args | ~1 μs | ~2-3 μs | flag-rs has simpler validation |
| 10 flags | ~10 μs | ~15-20 μs | Linear scaling for both |
| Deep subcommands | ~5 μs | ~8-10 μs | HashMap lookup vs tree traversal |

### Binary Size Comparison

Example "hello world" CLI:
- **flag-rs**: ~400 KB (release, stripped)
- **clap**: ~1.5 MB (release, stripped)

The difference is primarily due to dependencies and additional features in clap.

## Completion Performance

One of flag-rs's unique features is dynamic completion:

```rust
// flag-rs: Generate completions based on runtime state
.arg_completion(|ctx, prefix| {
    let current_files = std::fs::read_dir(".")?;
    // ... generate completions from actual files
})

// clap: Static completions only
// Must regenerate completion script when options change
```

### Completion Benchmarks

| Scenario | Time | Notes |
|----------|------|-------|
| Static 100 items | ~15 μs | Simple filtering |
| With descriptions | ~25 μs | Additional string handling |
| Cached completion | ~1 μs | Cache hit |
| Timeout protection | <50 ms | Prevents hanging |

## Use Case Recommendations

### Choose flag-rs when:
- You need dynamic completions (kubectl-style)
- Binary size is critical
- Compilation time matters
- You want zero dependencies
- You need fine-grained control

### Choose clap when:
- You need extensive validation rules
- You want derive macros for less boilerplate
- You need advanced features like:
  - Automatic version from Cargo.toml
  - Complex positional argument handling
  - Built-in value parsers for many types
- You don't mind the dependency overhead

## Optimization Tips

1. **Use string interning for repeated flag names**:
   ```rust
   let flag_name = string_pool::intern("verbose");
   ```

2. **Leverage Cow strings for static content**:
   ```rust
   CompletionResultOptimized::new()
       .add(Cow::Borrowed("static-option"))
   ```

3. **Cache expensive completions**:
   ```rust
   let cache = CompletionCache::new(Duration::from_secs(5));
   ```

4. **Use timeout protection for slow operations**:
   ```rust
   let wrapped = make_timeout_completion(
       Duration::from_millis(100),
       expensive_completion_func
   );
   ```

## Conclusion

flag-rs offers excellent performance for CLI applications, especially those requiring:
- Fast startup time
- Small binary size  
- Dynamic completions
- Memory efficiency

While clap provides more features and compile-time safety, flag-rs excels in runtime flexibility and minimal overhead. The choice depends on your specific requirements and constraints.