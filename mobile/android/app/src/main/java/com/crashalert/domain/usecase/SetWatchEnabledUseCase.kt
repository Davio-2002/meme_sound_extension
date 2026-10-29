package com.crashalert.domain.usecase

import com.crashalert.domain.repository.PermissionRepository
import com.crashalert.domain.repository.SettingsRepository

// Enables monitoring only if the required permissions are granted. Returns the result.
class SetWatchEnabledUseCase(
    private val permissions: PermissionRepository,
    private val settings: SettingsRepository
) {
    operator fun invoke(enabled: Boolean): Boolean {
        val result = enabled && permissions.canReadLogs()
        settings.watchEnabled = result
        return result
    }
}
