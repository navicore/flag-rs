# Roadmap

## Current State

Flag-rs is a working CLI framework at v0.8.4, published on crates.io. Core features are stable: command trees, flag parsing, dynamic completions, shell script generation (bash/zsh/fish), and performance utilities (caching, timeouts, string interning).

## Known Gaps / Potential Next Steps

- No async support - run functions are synchronous only
- No built-in config file integration (by design - zero dependencies)
- No PowerShell or Nushell completion support
- The `completion_optimized` and `parse_optimized` modules exist alongside the standard paths; unclear if they're meant to replace them eventually
- No derive macro / proc-macro for declarative command definitions (builder API is the only interface)
