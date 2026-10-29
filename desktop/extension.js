const vscode = require('vscode');
const path = require('path');
const { spawn } = require('child_process');

// Windows: one long-lived PowerShell with MediaPlayer (.mp3 and .wav) already loaded, so playing
// skips process startup. Reads lines from stdin: "o <path>" opens a file, "p <path>" plays it from the start.
const PLAYER_SCRIPT = [
  '[Console]::InputEncoding = [Text.Encoding]::UTF8',
  'Add-Type -AssemblyName PresentationCore',
  '$p = New-Object System.Windows.Media.MediaPlayer',
  '$cur = $null',
  'while ($null -ne ($line = [Console]::In.ReadLine())) {',
  '  $f = $line.Substring(2)',
  '  if ($f -ne $cur) { $p.Open([uri]$f); $cur = $f }',
  "  if ($line[0] -eq 'p') { $p.Stop(); $p.Play() }",
  '}',
].join('\n');

let player;

function send(command, file) {
  if (!player || player.exitCode !== null) {
    player = spawn('powershell.exe', ['-NoProfile', '-NonInteractive', '-Command', PLAYER_SCRIPT], {
      stdio: ['pipe', 'ignore', 'ignore'],
      windowsHide: true,
    });
    player.on('error', (err) => console.error('faaah: player failed', err));
    player.stdin.on('error', (err) => console.error('faaah: player stdin failed', err));
  }
  player.stdin.write(`${command} ${file}\n`);
}

function play(file) {
  if (process.platform === 'win32') {
    send('p', file);
    return;
  }
  const cmd = process.platform === 'darwin' ? 'afplay' : 'paplay';
  const child = spawn(cmd, [file], { stdio: 'ignore' });
  child.on('error', (err) => console.error('faaah: playback failed', err));
}

function activate(context) {
  const bundled = path.join(context.extensionPath, 'media', 'faaah.wav');
  const soundFile = () => vscode.workspace.getConfiguration('faaah').get('soundFile') || bundled;

  if (process.platform === 'win32') send('o', soundFile());

  context.subscriptions.push(
    { dispose: () => player && player.kill() },
    // Requires terminal shell integration (on by default for bash, zsh, fish, pwsh).
    vscode.window.onDidEndTerminalShellExecution((e) => {
      if (e.exitCode === undefined || e.exitCode === 0) return;
      play(soundFile());
    })
  );
}

function deactivate() {}

module.exports = { activate, deactivate };
