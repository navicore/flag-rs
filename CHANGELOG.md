# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

#### Phase 4 - Enhanced Help & UX
- **Enhanced Error Messages**:
  - `FlagParsing` error now includes flag name and suggestions
  - Better error messages for all flag type validations
  - Helpful suggestions for boolean, choice, range, file, and directory errors
  - Created `enhanced_errors_demo.rs` example
- **Better Help Formatting**:
  - Separate sections for required and optional flags
  - Display flag constraints inline (RequiredIf, ConflictsWith, Requires)
  - Show argument requirements in usage line
  - Enhanced visual hierarchy with better spacing
  - Created `help_formatting_demo.rs` example
- **Command Grouping Support**:
  - Already implemented - commands can be grouped with `group_id`
  - Groups displayed in help output with visual separation

#### Phase 5 - Advanced Flag Features
- Added new flag types:
  - `StringArray` - Array of string values
  - `Choice(Vec<String>)` - Enumerated choices with validation
  - `Range(i64, i64)` - Integer range validation
  - `File` - File path validation
  - `Directory` - Directory path validation
- Added flag constraint system:
  - `RequiredIf(String)` - Flag required if another flag is set
  - `ConflictsWith(Vec<String>)` - Flag conflicts with other flags
  - `Requires(Vec<String>)` - Flag requires other flags to be set
- Enhanced flag validation with detailed error messages

#### Phase 6 - Performance & Polish
- **Completion Caching** (`completion_cache.rs`):
  - Time-based cache with configurable TTL
  - Thread-safe implementation using Arc<Mutex<>>
  - Automatic cleanup of expired entries
  - Key generation based on context, prefix, and flags
- **Timeout Protection** (`completion_timeout.rs`):
  - Protect against slow completion functions
  - Configurable timeout duration
  - Extension trait for wrapping completion functions
  - Graceful degradation with timeout warnings
- **Memory Optimizations**:
  - String interning for repeated strings (`string_pool.rs`)
  - Optimized completion structures using Cow strings (`completion_optimized.rs`)
  - Memory-efficient CompletionItem struct
  - Reduced allocations in hot paths
- **Comprehensive Test Suite**:
  - Shell completion basic tests (10 tests)
  - Edge case tests including Unicode and special characters (11 tests)
  - Performance and thread safety tests (9 tests)
- **Performance Benchmarks**:
  - Comprehensive benchmark suite (`benchmark.rs`)
  - Detailed performance documentation (`PERFORMANCE.md`)
  - Comparison with clap characteristics

### Changed
- Updated parse_value to support new flag types
- Enhanced validation to check flag constraints
- Improved error messages for validation failures

### Fixed
- Resolved segfault in flag constraint validation due to unsafe parent pointers
- Fixed multiple clippy lints throughout the codebase
- Corrected Result type usage in timeout module

## [0.7.1] - Previous release

### Changed
- Version bump for maintenance

## [0.7.0] - Previous release

### Added
- Initial implementation of core CLI framework
- Dynamic completion support
- Shell completion generation for bash, zsh, fish
- Modular command architecture
- Flag inheritance system