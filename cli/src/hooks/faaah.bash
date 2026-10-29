# faaah hook for bash 4.4+: eval "$(faaah init bash)"

__faaah_hook() {
    local exit_code=$?
    if [[ -n $__faaah_ran ]] && ((exit_code != 0)); then
        faaah play --detach --exit-code="$exit_code"
    fi
    __faaah_ran=
    return "$exit_code"
}

if [[ $PROMPT_COMMAND != *__faaah_hook* ]]; then
    PROMPT_COMMAND="__faaah_hook${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
    # PS0 expands only when a command is about to run, so an empty Enter does not count.
    PS0='${__faaah_ran:0:$((__faaah_ran = 1, 0))}'"$PS0"
fi
