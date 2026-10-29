package com.crashalert.data

import android.content.Context
import androidx.core.content.edit
import com.crashalert.domain.repository.SettingsRepository

class PrefsSettingsRepository(context: Context) : SettingsRepository {

    private val prefs = context.getSharedPreferences("settings", Context.MODE_PRIVATE)

    override var watchEnabled: Boolean
        get() = prefs.getBoolean(KEY_ENABLED, false)
        set(value) = prefs.edit { putBoolean(KEY_ENABLED, value) }

    private companion object {
        const val KEY_ENABLED = "watch_enabled"
    }
}
