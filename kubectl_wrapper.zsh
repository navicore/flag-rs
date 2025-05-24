#!/usr/bin/env zsh

# Function wrapper for kubectl
kubectl() {
    ./target/debug/examples/kubectl "$@"
}

# Completion function
_kubectl() {
    local -a completions
    local IFS=$'\n'
    
    # Build completion arguments
    local -a comp_line
    comp_line=("__complete")
    
    # Add all previous words
    if (( CURRENT > 1 )); then
        comp_line+=("${words[@]:1:$((CURRENT-1))}")
    fi
    
    # Add current word
    comp_line+=("$words[CURRENT]")
    
    # Get completions
    local response
    response=$(KUBECTL_COMPLETE=zsh ./target/debug/examples/kubectl "${comp_line[@]}" 2>/dev/null)
    
    if [[ -n "$response" ]]; then
        completions=(${(f)response})
        _describe 'kubectl commands' completions
    fi
}

# Register completion
compdef _kubectl kubectl