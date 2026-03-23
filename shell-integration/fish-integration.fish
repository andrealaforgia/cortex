#!/usr/bin/env fish
# AI Terminal - Fish Shell Integration
# Emits OSC 133 sequences for block detection

function __aiterminal_prompt --on-event fish_prompt
    printf '\e]133;A\a'
end

function __aiterminal_preexec --on-event fish_preexec
    printf '\e]133;C\a'
end

function __aiterminal_postexec --on-event fish_postexec
    printf '\e]133;D;%s\a' $status
end

# Emit initial prompt start
printf '\e]133;A\a'
