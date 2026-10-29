package com.crashalert.ui

import androidx.lifecycle.ViewModel
import com.crashalert.domain.model.CrashType
import com.crashalert.domain.model.SetupStatus
import com.crashalert.domain.usecase.GetSetupStatusUseCase
import com.crashalert.domain.usecase.PlayTestAlertUseCase
import com.crashalert.domain.usecase.SetWatchEnabledUseCase
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

class MainViewModel(
    private val getSetupStatus: GetSetupStatusUseCase,
    private val setWatchEnabled: SetWatchEnabledUseCase,
    private val playTestAlert: PlayTestAlertUseCase
) : ViewModel() {

    private val _state = MutableStateFlow(getSetupStatus())
    val state: StateFlow<SetupStatus> = _state.asStateFlow()

    // Permissions may have been granted via adb while the screen was in background.
    fun refresh() {
        _state.value = getSetupStatus()
    }

    fun onToggle(enabled: Boolean) {
        setWatchEnabled(enabled)
        refresh()
    }

    fun onTestSound(type: CrashType) = playTestAlert(type)
}
