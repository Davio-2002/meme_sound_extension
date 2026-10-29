# faaah hook for fish: faaah init fish | source

function __faaah_postexec --on-event fish_postexec
    set -l exit_code $status
    if test $exit_code -ne 0
        faaah play --detach --exit-code=$exit_code
    end
end
