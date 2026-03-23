#!/bin/bash
# AI Terminal - Bash Shell Integration
# Emits OSC 133 sequences for block detection

__aiterminal_prompt_command() {
    local exit_code=$?
    # OSC 133;D -- previous command finished with exit code
    printf '\e]133;D;%s\a' "$exit_code"
    # OSC 133;A -- prompt start
    printf '\e]133;A\a'
}

__aiterminal_preexec() {
    # OSC 133;C -- command executed
    printf '\e]133;C\a'
}

# OSC 133;B is emitted after PS1 is displayed (prompt end / command start)
if [[ -z "$PROMPT_COMMAND" ]]; then
    PROMPT_COMMAND="__aiterminal_prompt_command"
else
    PROMPT_COMMAND="__aiterminal_prompt_command;$PROMPT_COMMAND"
fi

trap '__aiterminal_preexec' DEBUG

# Emit initial prompt start
printf '\e]133;A\a'
