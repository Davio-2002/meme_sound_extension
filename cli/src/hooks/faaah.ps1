# faaah hook for PowerShell: faaah init powershell | Out-String | Invoke-Expression

if (-not $global:__FaaahPrompt) {
    $global:__FaaahPrompt = $function:prompt
    $global:__FaaahHistoryId = (Get-History -Count 1).Id

    function global:prompt {
        $succeeded = $?
        $exitCode = $global:LASTEXITCODE
        # History grows only when a command ran, so an empty Enter does not count.
        $historyId = (Get-History -Count 1).Id
        if ($historyId -ne $global:__FaaahHistoryId) {
            $global:__FaaahHistoryId = $historyId
            if (-not $succeeded) {
                # $LASTEXITCODE is empty until the first external program runs.
                $playArgs = @('play', '--detach')
                if ($null -ne $exitCode) {
                    $playArgs += "--exit-code=$exitCode"
                }
                faaah @playArgs
                $global:LASTEXITCODE = $exitCode
            }
        }
        # Give the original prompt the same $? the user's command left.
        if (-not $succeeded) {
            Write-Error '' -ErrorAction Ignore
        }
        & $global:__FaaahPrompt
    }
}
