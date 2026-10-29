# Faaah

Plays a meme sound when something fails.

| Module | Platform | Trigger | Status |
|---|---|---|---|
| [`cli/`](cli) | Windows, macOS, Linux | A terminal command exits with a non-zero code | In progress |
| [`desktop/hooks/`](desktop/hooks) | Windows | A terminal command exits with a non-zero code | Legacy, replaced by `cli/` |
| [`mobile/android/`](mobile/android) | Android 8.0+ | Any app crashes or hangs (ANR), or the system fails | Works |

---

## CLI

A single `faaah` binary with the sound built in. It plays audio itself, so no external player is needed.

### Build

Needs Rust (stable). On Linux also install the ALSA headers, for example `sudo apt install libasound2-dev`.

```bash
cd cli
cargo build --release
```

The binary is `cli/target/release/faaah` (`faaah.exe` on Windows).

### Commands

| Command | What it does |
|---|---|
| `faaah play` | Plays the sound and waits until it ends. Does nothing when the sound is off |
| `faaah play --detach` | Returns at once, the sound plays in a background process |
| `faaah test` | Plays the sound and reports an error if audio does not work |
| `faaah sound <file>` | Uses a `.wav` or `.mp3` file instead of the built-in sound |
| `faaah sound --reset` | Goes back to the built-in sound |
| `faaah on` / `faaah off` | Turns the sound on or off |
| `faaah status` | Shows whether the sound is on, which file plays, and where the config is |
| `faaah init <shell>` | Prints the hook for `bash`, `zsh`, `fish`, `powershell` or `cmd` |
| `faaah install [shell...]` | Adds the hook to shell configs; without arguments, to every shell found |
| `faaah uninstall [shell...]` | Removes the hook; without arguments, from every shell |

Settings are stored in `config.toml` inside the `faaah` folder of the user config directory: `~/.config` on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows.

`ignore_codes` in the config lists exit codes that stay silent. By default it is Ctrl+C: `130` on Unix and `-1073741510` on Windows.

### Shell hooks

Put `faaah` on `PATH` and run:

```bash
faaah install
```

It finds bash, zsh, fish, PowerShell and cmd with Clink, and adds a marked block to each config file. Running it again changes nothing; `faaah uninstall` removes the block and leaves the rest of the file as it was. The sound plays only after a command that really ran and failed; an empty Enter does not count.

To set it up by hand, add one line to the shell config:

| Shell | File | Line |
|---|---|---|
| bash 4.4+ | `~/.bashrc` | `eval "$(faaah init bash)"` |
| zsh | `~/.zshrc` | `eval "$(faaah init zsh)"` |
| fish | `~/.config/fish/config.fish` | `faaah init fish \| source` |
| PowerShell 7 and 5.1 | `$PROFILE` (at the end) | `faaah init powershell \| Out-String \| Invoke-Expression` |
| cmd with [Clink](https://chrisant996.github.io/clink/) | `faaah.lua` in the Clink scripts folder (`clink info` shows it) | `load(io.popen("faaah init cmd"):read("*a"))()` |

### Development

```bash
cd cli
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

---

## Shell hooks (Windows, legacy)

Manual setup until `faaah install` is ready. The hooks play the sound through Windows `MediaPlayer`, so they need `powershell.exe`.

Before installing any hook, replace the placeholder `faah_path` inside the hook file with the absolute Windows path to the sound, for example `C:\tools\meme_sound_extension\cli\assets\faaah.wav`.

### PowerShell (7 and 5.1)

1. Replace `faah_path` in [`desktop/hooks/powershell/faaah.ps1`](desktop/hooks/powershell/faaah.ps1).
2. Add this line to your profile (open it with `notepad $PROFILE`). Do it for both PowerShell 7 and 5.1 if you use both:

   ```powershell
   . "C:\path\to\meme_sound_extension\desktop\hooks\powershell\faaah.ps1"
   ```

3. Open a new window and run a failing command, for example `Get-Item nope`.

### Bash (Git Bash on Windows)

1. Replace `faah_path` in [`desktop/hooks/bash/faaah.bashrc`](desktop/hooks/bash/faaah.bashrc) with a Windows path (it is opened by `powershell.exe`).
2. Add this line to `~/.bashrc`:

   ```bash
   source /c/path/to/meme_sound_extension/desktop/hooks/bash/faaah.bashrc
   ```

3. Open a new terminal and run `false`.

### cmd (needs [Clink](https://chrisant996.github.io/clink/))

1. Replace `faah_path` in [`desktop/hooks/cmd/faaah-player.ps1`](desktop/hooks/cmd/faaah-player.ps1).
2. Register the hooks folder as a Clink scripts folder (`faaah.lua` looks for `faaah-player.ps1` next to itself):

   ```bat
   clink installscripts "C:\path\to\meme_sound_extension\desktop\hooks\cmd"
   ```

3. Open a new cmd window and run a failing command, for example `dir nope`.

---

## Mobile (Android)

An Android app that plays [`crash_sound.wav`](mobile/android/app/src/main/res/raw/crash_sound.wav) when:

- any app crashes (Java or native),
- any app hangs (ANR),
- the system fails (`system_server` crash, watchdog, restart after a kernel failure).

It listens to the system crash log (`DropBoxManager`) from a foreground service, so it keeps working in the background and starts again after reboot. The sound uses the alarm stream, so you hear it even when the ringer is silent.

### Requirements

- Android 8.0+ (API 26). Tested on Pixel 8a, Android 16.
- To build: JDK 17+ (the JDK bundled with Android Studio works) and Android SDK 37.
- A computer with `adb` and USB debugging on the phone, once, to grant permissions.

### Build and install

From `mobile/android`. On Windows use `gradlew.bat` instead of `./gradlew`.

```bash
./gradlew assembleDebug
```

```bash
adb install -r app/build/outputs/apk/debug/app-debug.apk
```

Gradle needs to know where the Android SDK is. Android Studio sets this up by itself; on the command line set `ANDROID_HOME` or put `sdk.dir=...` into `mobile/android/local.properties`.

You can also open `mobile/android` in Android Studio and press **Run**.

### Grant permissions (once)

Android does not let a normal app see other apps' crashes. The app needs two protected permissions, and only `adb` can grant them:

```bash
adb shell pm grant com.crashalert android.permission.READ_LOGS
```

```bash
adb shell pm grant com.crashalert android.permission.READ_DROPBOX_DATA
```

`READ_DROPBOX_DATA` is needed on Android 15+. The grant survives reboots and app updates, but not reinstalling after uninstall.

> **Security note:** these permissions give the app access to system logs, which may contain data of other apps. The app reads only the entry tag (crash type), not the content, and has no network access. Grant them only to a build you trust.

### Use

1. Open **Crash Notifier**. The screen shows "Permissions granted".
2. Turn on **Play sound on crashes**. A permanent notification shows that monitoring is on.
3. The **Test sound** buttons play the sound without a real crash.

Test with a real crash:

```bash
adb shell am crash com.android.settings
```

If a burst of failures happens, the sound plays at most once per 4 seconds.

### Run unit tests

```bash
./gradlew testDebugUnitTest
```

### Limitations

- Some vendor ROMs (MIUI, ColorOS and others) may kill background services or skip some crash log entries. Allow the app to run in the background in battery settings.
- Without the two permissions the app cannot work; there is no other way on non-rooted Android.

---

## Project layout

```
cli/                    faaah CLI (Rust)
  src/                  commands, config, sound playback
  src/hooks/            shell hooks printed by `faaah init`
  assets/faaah.wav      bundled sound
desktop/hooks/          legacy Windows hooks (PowerShell, bash, cmd)
mobile/android/
  app/src/main/java/com/crashalert/
    ui/                 screen and ViewModel
    domain/             models, use cases, repository interfaces
    data/               DropBox source, sound player, settings, permissions
    service/            foreground service and boot receiver
  app/src/test/         unit tests
```
