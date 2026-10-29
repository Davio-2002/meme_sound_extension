# faaah: play a sound when a command fails. Skipped in VS Code terminals, where the Faaah extension plays it.
if [[ $- == *i* && $TERM_PROGRAM != vscode ]]; then
    __faaah_file='C:\Users\Khachatryan.Davit.PROTELION\Downloads\meme-sound-effect-trimmed.wav'
    # One long-lived PowerShell with the clip loaded, so playing skips process startup; each stdin line restarts playback.
    coproc __FAAAH { powershell.exe -NoProfile -NonInteractive -Command "Add-Type -AssemblyName PresentationCore; \$p = New-Object System.Windows.Media.MediaPlayer; \$p.Open([uri]'$__faaah_file'); while (\$null -ne [Console]::In.ReadLine()) { \$p.Stop(); \$p.Play() }"; }
    __faaah_play() { [[ -n ${__FAAAH[1]} ]] && echo >&"${__FAAAH[1]}"; }
    __faaah_last=
    __faaah() {
        local status=$?
        # Only react when a new command ran, so an empty Enter doesn't replay the sound.
        if [[ -n $__faaah_last && $HISTCMD != "$__faaah_last" ]] && (( status != 0 )); then
            __faaah_play
        fi
        __faaah_last=$HISTCMD
        return $status
    }
    PROMPT_COMMAND="__faaah${PROMPT_COMMAND:+; $PROMPT_COMMAND}"
fi
