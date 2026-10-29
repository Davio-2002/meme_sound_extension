# faaah: play a sound when a command fails. Dot-sourced from the PowerShell 7 and 5.1 profiles.
# Skipped in VS Code terminals, where the Faaah extension already plays it.
if ($env:TERM_PROGRAM -ne 'vscode') {
    Add-Type -AssemblyName PresentationCore
    $global:FaaahPlayer = New-Object System.Windows.Media.MediaPlayer
    $global:FaaahPlayer.Open([uri]'faah_path')
    $global:FaaahLastId = (Get-History -Count 1).Id
    $global:FaaahPrompt = $function:prompt

    function global:prompt {
        $ok = $?
        # Only react when a new command ran, so an empty Enter doesn't replay the sound.
        $id = (Get-History -Count 1).Id
        if ($id -ne $global:FaaahLastId) {
            $global:FaaahLastId = $id
            if (-not $ok) {
                $global:FaaahPlayer.Stop()
                $global:FaaahPlayer.Play()
            }
        }
        & $global:FaaahPrompt
    }
}
