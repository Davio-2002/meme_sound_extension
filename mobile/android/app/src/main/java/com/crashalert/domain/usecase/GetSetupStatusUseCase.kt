package com.crashalert.domain.usecase

import com.crashalert.domain.model.SetupStatus
import com.crashalert.domain.repository.PermissionRepository
import com.crashalert.domain.repository.SettingsRepository

class GetSetupStatusUseCase(
    private val permissions: PermissionRepository,
    private val settings: SettingsRepository
) {
    operator fun invoke() = SetupStatus(
        canReadLogs = permissions.canReadLogs(),
        watchEnabled = settings.watchEnabled
    )
}
