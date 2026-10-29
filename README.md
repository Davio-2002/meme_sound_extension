# Meme Sound Extension

Plays a meme sound when something fails:

| Module | Platform | Trigger |
|---|---|---|
| [`desktop/`](desktop) | Shell hooks (Windows) | A terminal command exits with a non-zero code |
| [`mobile/android/`](mobile/android) | Android 8.0+ | Any app crashes or hangs (ANR), or the system fails |

---

## Desktop

Shell hooks for Windows terminals (Windows Terminal, VS Code, PowerShell, Git Bash, cmd).

### Shell hooks (Windows only)

The hooks play the sound through Windows `MediaPlayer`, so they need `powershell.exe`.

Before installing any hook, replace the placeholder `faah_path` inside the hook file with the absolute Windows path to the sound, for example `C:\tools\meme_sound_extension\cli\assets\faaah.wav`.

#### PowerShell (7 and 5.1)

1. Replace `faah_path` in [`desktop/hooks/powershell/faaah.ps1`](desktop/hooks/powershell/faaah.ps1).
2. Add this line to your profile (open it with `notepad $PROFILE`). Do it for both PowerShell 7 and 5.1 if you use both:

   ```powershell
   . "C:\path\to\meme_sound_extension\desktop\hooks\powershell\faaah.ps1"
   ```

3. Open a new window and run a failing command, for example `Get-Item nope`.

#### Bash (Git Bash on Windows)

1. Replace `faah_path` in [`desktop/hooks/bash/faaah.bashrc`](desktop/hooks/bash/faaah.bashrc) with a Windows path (it is opened by `powershell.exe`).
2. Add this line to `~/.bashrc`:

   ```bash
   source /c/path/to/meme_sound_extension/desktop/hooks/bash/faaah.bashrc
   ```

3. Open a new terminal and run `false`.

#### cmd (needs [Clink](https://chrisant996.github.io/clink/))

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
cli/                    faaah CLI (Rust, work in progress)
  src/                  commands and sound playback
  assets/faaah.wav      bundled sound
desktop/
  hooks/powershell/     PowerShell profile hook
  hooks/bash/           bash hook
  hooks/cmd/            cmd hook (Clink script + player)
mobile/android/
  app/src/main/java/com/crashalert/
    ui/                 screen and ViewModel
    domain/             models, use cases, repository interfaces
    data/               DropBox source, sound player, settings, permissions
    service/            foreground service and boot receiver
  app/src/test/         unit tests
```
