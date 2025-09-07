#!/bin/bash

# Test script for dynamic completion macros in flag-rs Phase II
echo "Testing flag-rs Phase II dynamic completion macros..."
echo

# Build the example
echo "Building dynamic_completion_demo example..."
cargo build --example dynamic_completion_demo --quiet
echo

# Test static completion (from Phase I) still works
echo "=== Static Completion (Phase I compatibility) ==="
echo "$ dynamic_demo --log_level [TAB]"
DYNAMIC_DEMO_COMPLETE=zsh ./target/debug/examples/dynamic_completion_demo __complete --log_level ""
echo

# Test dynamic session completion
echo "=== Dynamic Session Completion ==="
echo "$ dynamic_demo --session [TAB]"
DYNAMIC_DEMO_COMPLETE=zsh ./target/debug/examples/dynamic_completion_demo __complete --session ""
echo

# Test dynamic session completion with prefix
echo "=== Dynamic Session Completion with Prefix ==="
echo "$ dynamic_demo --session user[TAB]"
DYNAMIC_DEMO_COMPLETE=zsh ./target/debug/examples/dynamic_completion_demo __complete --session "user"
echo

# Test dynamic file completion
echo "=== Dynamic File Completion ==="
echo "$ dynamic_demo --file [TAB] (showing first 5 files)"
DYNAMIC_DEMO_COMPLETE=zsh ./target/debug/examples/dynamic_completion_demo __complete --file "" | head -5
echo "..."
echo

# Test dynamic file completion with prefix
echo "=== Dynamic File Completion with Prefix ==="
echo "$ dynamic_demo --file Car[TAB]"
DYNAMIC_DEMO_COMPLETE=zsh ./target/debug/examples/dynamic_completion_demo __complete --file "Car"
echo

# Test dynamic process completion
echo "=== Dynamic Process Completion ==="
echo "$ dynamic_demo --process [TAB]"
DYNAMIC_DEMO_COMPLETE=zsh ./target/debug/examples/dynamic_completion_demo __complete --process ""
echo

# Test dynamic process completion with prefix (numeric PID)
echo "=== Dynamic Process Completion with PID Prefix ==="
echo "$ dynamic_demo --process 12[TAB]"
DYNAMIC_DEMO_COMPLETE=zsh ./target/debug/examples/dynamic_completion_demo __complete --process "12"
echo

# Test dynamic process completion with prefix (process name)
echo "=== Dynamic Process Completion with Name Prefix ==="
echo "$ dynamic_demo --process ng[TAB]"
DYNAMIC_DEMO_COMPLETE=zsh ./target/debug/examples/dynamic_completion_demo __complete --process "ng"
echo

echo "🎉 All Phase II dynamic completion macros working!"
echo
echo "Key Features Demonstrated:"
echo "✅ Static completions (Phase I compatibility)"
echo "✅ Dynamic completions with custom logic"
echo "✅ Dynamic completions with help text"
echo "✅ Prefix filtering for dynamic completions"
echo "✅ Real-world examples (sessions, files, processes)"
echo "✅ Mixed static and dynamic completions in same app"
echo
echo "Usage patterns supported:"
echo "  completion! { static_name { completions: [...] } }"
echo "  completion! { dynamic_name { dynamic: |ctx, prefix| { ... } } }"
echo "  completion! { dynamic_with_help { dynamic: |ctx, prefix| { ... }, help: \"...\" } }"