-- faaah: play a sound when a command fails in cmd. Skipped in VS Code terminals, where the Faaah extension plays it.
if os.getenv("TERM_PROGRAM") == "vscode" then return end

-- Long-lived PowerShell player (faaah-player.ps1 next to this file), so playing skips process startup.
local script_dir = debug.getinfo(1, "S").source:match("^@(.*)[\\/]")
local player_cmd = 'powershell.exe -NoProfile -NonInteractive -File "' .. script_dir .. '\\faaah-player.ps1"'
local player = io.popen(player_cmd, "w")

local function play()
    for _ = 1, 2 do
        if not player then player = io.popen(player_cmd, "w") end
        if player and player:write("\n") and player:flush() then return end
        player = nil -- player died (e.g. Ctrl+C reaches it too); restart and retry once
    end
end

-- Only react when a command was actually entered, so an empty Enter doesn't replay the sound.
local ran = false
clink.onendedit(function(line) ran = line:match("%S") ~= nil end)
clink.onbeginedit(function()
    if ran and os.geterrorlevel() ~= 0 then play() end
    ran = false
end)
