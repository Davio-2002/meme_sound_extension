# Faaah

Plays a meme sound when something fails.

| Module | Platform | Trigger |
|---|---|---|
| [`cli/`](cli) | Windows, macOS, Linux | A terminal command exits with a non-zero code |
| [`mobile/android/`](mobile/android) | Android 8.0+ | Any app crashes or hangs (ANR), or the system fails |

---

## CLI

`faaah` is a single binary with the sound built in. It plays audio itself, so it needs no external player.

### Requirements

- [Rust](https://rustup.rs) (stable) to build.
- Linux: ALSA headers to build (`sudo apt install libasound2-dev` or the package of your distro).
- bash 4.4+ for the bash hook. macOS ships bash 3.2, so use zsh there or install a newer bash.
- cmd: [Clink](https://chrisant996.github.io/clink/), because plain cmd has no hooks.

### Install

```bash
cargo install --path cli
faaah install
faaah test
```

1. `cargo install` builds `faaah` and puts it into `~/.cargo/bin`, which rustup adds to `PATH`.
2. `faaah install` adds the hook to every shell it finds: bash, zsh, fish, PowerShell, and cmd with Clink.
3. `faaah test` plays the sound once, so you know audio works.

Open a new terminal and run a failing command, for example `git foo`.

### Commands

| Command | What it does |
|---|---|
| `faaah install [shell...]` | Adds the hook to shell configs; without arguments, to every shell found |
| `faaah uninstall [shell...]` | Removes the hook; without arguments, from every shell |
| `faaah on` / `faaah off` | Turns the sound on or off, hooks stay in place |
| `faaah sound <file>` | Uses a `.wav` or `.mp3` file instead of the built-in sound |
| `faaah sound --reset` | Goes back to the built-in sound |
| `faaah status` | Shows whether the sound is on, which file plays, which shells have the hook, and where the config is |
| `faaah test` | Plays the sound and reports an error if audio does not work |
| `faaah play [--detach]` | Plays the sound; `--detach` returns at once and plays in the background. Used by the hooks |
| `faaah init <shell>` | Prints the hook for `bash`, `zsh`, `fish`, `powershell` or `cmd` |

### Configuration

`config.toml` in the `faaah` folder of the user config directory: `~/.config` on Linux, `~/Library/Application Support` on macOS, `%APPDATA%` on Windows. `faaah status` prints the exact path.

```toml
muted = false
sound = "/home/me/sounds/bruh.mp3"
ignore_codes = [130, -1073741510]
```

| Key | Default | Meaning |
|---|---|---|
| `muted` | `false` | `true` keeps the hooks silent |
| `sound` | not set | Sound file; not set means the built-in sound |
| `ignore_codes` | `[130, -1073741510]` | Exit codes that stay silent. The defaults are Ctrl+C on Unix and on Windows |

### How the hooks work

After each command the hook checks its exit code. If the command really ran and failed, the hook calls `faaah play --detach`, which returns at once, so the prompt does not wait for the sound. An empty Enter does not count.

`faaah install` adds a marked block to the end of each config file. Running it again changes nothing, and `faaah uninstall` removes only that block:

```bash
# >>> faaah >>>
eval "$(faaah init bash)"
# <<< faaah <<<
```

To set it up by hand, add the line to the shell config:

| Shell | File | Line |
|---|---|---|
| bash 4.4+ | `~/.bashrc` | `eval "$(faaah init bash)"` |
| zsh | `~/.zshrc` | `eval "$(faaah init zsh)"` |
| fish | `~/.config/fish/config.fish` | `faaah init fish \| source` |
| PowerShell 7 and 5.1 | `$PROFILE` (at the end) | `faaah init powershell \| Out-String \| Invoke-Expression` |
| cmd with Clink | `%LOCALAPPDATA%\clink\faaah.lua` | `load(io.popen("faaah init cmd"):read("*a"))()` |

### Uninstall

```bash
faaah uninstall
cargo uninstall faaah
```

### Development

```bash
cd cli
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The hook syntax tests use `bash`, `zsh`, `fish` and `luac` when they are installed and skip the rest.

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
cli/
  src/main.rs           commands
  src/config.rs         config.toml
  src/sound.rs          sound playback
  src/hooks.rs          shells and their hooks
  src/hooks/            hook scripts printed by `faaah init`
  src/install.rs        adding and removing hooks in shell configs
  assets/faaah.wav      built-in sound
mobile/android/
  app/src/main/java/com/crashalert/
    ui/                 screen and ViewModel
    domain/             models, use cases, repository interfaces
    data/               DropBox source, sound player, settings, permissions
    service/            foreground service and boot receiver
  app/src/test/         unit tests
```
