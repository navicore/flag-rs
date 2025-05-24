# Zsh completion for kubectl
# Make sure completion system is initialized
autoload -Uz compinit && compinit

_kubectl_complete() {
    local -a completions
    local IFS=$'\n'
    
    # Get the actual kubectl command path
    local kubectl_cmd="${KUBECTL_PATH:-./target/debug/examples/kubectl}"
    
    # Build completion arguments
    local -a comp_line
    comp_line=("__complete")
    
    # Add all words except the command name
    local i
    for (( i = 2; i < CURRENT; i++ )); do
        comp_line+=("${words[$i]}")
    done
    
    # Add the current word being completed
    comp_line+=("${words[CURRENT]}")
    
    # Call the command with completion environment variable
    local response
    response=$(KUBECTL_COMPLETE=zsh "$kubectl_cmd" "${comp_line[@]}" 2>/dev/null)
    
    if [[ -n "$response" ]]; then
        completions=(${(f)response})
        compadd -a completions
    fi
}

# Register the completion function
compdef _kubectl_complete kubectl