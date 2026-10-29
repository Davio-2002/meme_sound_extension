# faaah player for cmd (started by faaah.lua): keeps the clip loaded; each line on stdin restarts playback.
Add-Type -AssemblyName PresentationCore
$p = New-Object System.Windows.Media.MediaPlayer
$p.Open([uri]'C:\Users\Khachatryan.Davit.PROTELION\Downloads\meme-sound-effect-trimmed.wav')
while ($null -ne [Console]::In.ReadLine()) { $p.Stop(); $p.Play() }
