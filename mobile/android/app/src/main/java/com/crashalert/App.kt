package com.crashalert

import android.app.Application
import com.crashalert.data.AndroidPermissionRepository
import com.crashalert.data.DropBoxCrashSource
import com.crashalert.data.PrefsSettingsRepository
import com.crashalert.data.SoundAlertPlayer
import com.crashalert.domain.usecase.GetSetupStatusUseCase
import com.crashalert.domain.usecase.HandleCrashTagUseCase
import com.crashalert.domain.usecase.PlayTestAlertUseCase
import com.crashalert.domain.usecase.SetWatchEnabledUseCase

// Manual DI: all dependencies are wired here.
class App : Application() {

    lateinit var graph: AppGraph
        private set

    override fun onCreate() {
        super.onCreate()
        graph = AppGraph(this)
    }
}

class AppGraph(app: Application) {
    private val player = SoundAlertPlayer(app)
    private val permissions = AndroidPermissionRepository(app)
    private val settings = PrefsSettingsRepository(app)

    val crashSource = DropBoxCrashSource(app)
    val handleCrashTag = HandleCrashTagUseCase(player)
    val getSetupStatus = GetSetupStatusUseCase(permissions, settings)
    val setWatchEnabled = SetWatchEnabledUseCase(permissions, settings)
    val playTestAlert = PlayTestAlertUseCase(player)
}

val android.content.Context.graph: AppGraph
    get() = (applicationContext as App).graph
