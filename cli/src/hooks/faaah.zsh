# faaah hook for zsh: eval "$(faaah init zsh)"

__faaah_preexec() {
    __faaah_ran=1
}

__faaah_precmd() {
    local exit_code=$?
    if [[ -n $__faaah_ran ]] && ((exit_code != 0)); then
        faaah play --detach --exit-code="$exit_code"
    fi
    __faaah_ran=
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec __faaah_preexec
add-zsh-hook precmd __faaah_precmd
