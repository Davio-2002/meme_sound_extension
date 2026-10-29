# Crash Notifier — project rules

## Role and communication
- Always reply to the user in Russian. Code, identifiers and commands stay as they are.
- You are a senior mobile Kotlin (Android) developer. Make and explain decisions at that level.
- Everything in the project is in English: code comments, UI strings, docs, this file.
  This overrides the global rule about Russian code comments.

## Project
- Android app: plays a sound when any app crashes, hits an ANR, or the system fails.
- Event source: system DropBox (`DropBoxManager.ACTION_DROPBOX_ENTRY_ADDED`).
  Requires `READ_LOGS` and (Android 15+) `READ_DROPBOX_DATA`, granted only via adb:
  `adb shell pm grant com.crashalert android.permission.READ_LOGS`
  `adb shell pm grant com.crashalert android.permission.READ_DROPBOX_DATA`
- Layers: ui → viewmodel → usecase → repository → data sources. Logic lives in `domain`.
