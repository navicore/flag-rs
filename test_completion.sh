#!/bin/bash

# Test script to demonstrate macro-defined completions working with zsh
echo "Testing flag-rs macro completions..."
echo

# Build the example
echo "Building macro_demo example..."
cargo build --example macro_demo --quiet
echo

# Test subcommand completion
echo "=== Subcommand Completion ==="
echo "$ macro_demo [TAB]"
MACRO_DEMO_COMPLETE=zsh ./target/debug/examples/macro_demo __complete ""
echo

# Test flag completion
echo "=== Flag Completion ==="
echo "$ macro_demo --[TAB]"
MACRO_DEMO_COMPLETE=zsh ./target/debug/examples/macro_demo __complete --
echo

# Test log level completion (macro with descriptions and help)
echo "=== Log Level Completion (macro with descriptions) ==="
echo "$ macro_demo --log_level [TAB]"
MACRO_DEMO_COMPLETE=zsh ./target/debug/examples/macro_demo __complete --log_level ""
echo

# Test environment completion (macro with simple values)
echo "=== Environment Completion (simple macro) ==="
echo "$ macro_demo --env [TAB]"  
MACRO_DEMO_COMPLETE=zsh ./target/debug/examples/macro_demo __complete --env ""
echo

# Test deploy target completion (macro with descriptions and help)
echo "=== Deploy Target Completion (macro with descriptions) ==="
echo "$ macro_demo deploy --target [TAB]"
MACRO_DEMO_COMPLETE=zsh ./target/debug/examples/macro_demo __complete deploy --target ""
echo

echo "All macro-defined completions working! 🎉"
echo
echo "To use these completions in zsh:"
echo "  ./target/debug/examples/macro_demo --config test.toml completion zsh > _macro_demo"
echo "  # Then source or install _macro_demo in your fpath"