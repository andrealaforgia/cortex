#!/bin/zsh
# AI Terminal - Zsh Shell Integration
# Emits OSC 133 sequences for block detection

__aiterminal_precmd() {
    local exit_code=$?
    # OSC 133;D -- previous command finished with exit code
    print -Pn '\e]133;D;%s\a' "$exit_code"
    # OSC 133;A -- prompt start
    print -Pn '\e]133;A\a'
}

__aiterminal_preexec() {
    # OSC 133;C -- command executed
    print -Pn '\e]133;C\a'
}

precmd_functions+=(__aiterminal_precmd)
preexec_functions+=(__aiterminal_preexec)

# Emit initial prompt start
print -Pn '\e]133;A\a'
