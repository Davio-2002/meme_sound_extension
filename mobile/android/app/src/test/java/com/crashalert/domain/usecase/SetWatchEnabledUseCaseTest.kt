package com.crashalert.domain.usecase

import com.crashalert.domain.repository.PermissionRepository
import com.crashalert.domain.repository.SettingsRepository
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SetWatchEnabledUseCaseTest {

    private class FakePermissions(var granted: Boolean) : PermissionRepository {
        override fun canReadLogs() = granted
    }

    private class FakeSettings : SettingsRepository {
        override var watchEnabled = false
    }

    private val permissions = FakePermissions(granted = true)
    private val settings = FakeSettings()
    private val setEnabled = SetWatchEnabledUseCase(permissions, settings)

    @Test
    fun `enables when permission granted`() {
        assertTrue(setEnabled(true))
        assertTrue(settings.watchEnabled)
    }

    @Test
    fun `stays off without permission`() {
        permissions.granted = false
        assertFalse(setEnabled(true))
        assertFalse(settings.watchEnabled)
    }

    @Test
    fun `disables`() {
        setEnabled(true)
        assertFalse(setEnabled(false))
        assertFalse(settings.watchEnabled)
    }

    @Test
    fun `status reflects permission and setting`() {
        setEnabled(true)
        val status = GetSetupStatusUseCase(permissions, settings)()
        assertTrue(status.canReadLogs)
        assertTrue(status.watchEnabled)
    }
}
