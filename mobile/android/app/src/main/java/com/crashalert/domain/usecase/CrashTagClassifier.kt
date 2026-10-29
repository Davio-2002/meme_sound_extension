package com.crashalert.domain.usecase

import com.crashalert.domain.model.CrashType

// Maps a DropBox entry tag to a crash type. Unknown tag -> null.
object CrashTagClassifier {

    private val tags = mapOf(
        "data_app_crash" to CrashType.APP_CRASH,
        "system_app_crash" to CrashType.APP_CRASH,
        "data_app_native_crash" to CrashType.NATIVE_CRASH,
        "system_app_native_crash" to CrashType.NATIVE_CRASH,
        "SYSTEM_TOMBSTONE" to CrashType.NATIVE_CRASH,
        "data_app_anr" to CrashType.ANR,
        "system_app_anr" to CrashType.ANR,
        "system_server_crash" to CrashType.SYSTEM,
        "system_server_native_crash" to CrashType.SYSTEM,
        "system_server_anr" to CrashType.SYSTEM,
        "system_server_watchdog" to CrashType.SYSTEM,
        "SYSTEM_RESTART" to CrashType.SYSTEM,
        "SYSTEM_LAST_KMSG" to CrashType.SYSTEM
    )

    fun classify(tag: String?): CrashType? = tag?.let { tags[it] }
}
