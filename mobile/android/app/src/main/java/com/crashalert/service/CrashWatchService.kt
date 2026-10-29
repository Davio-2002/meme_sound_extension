package com.crashalert.service

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import androidx.core.app.NotificationCompat
import androidx.core.app.ServiceCompat
import androidx.core.content.ContextCompat
import androidx.lifecycle.LifecycleService
import androidx.lifecycle.lifecycleScope
import com.crashalert.R
import com.crashalert.graph
import kotlinx.coroutines.Job
import kotlinx.coroutines.launch

// Foreground service: listens to the crash log and plays the alert sound.
class CrashWatchService : LifecycleService() {

    private var watchJob: Job? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        super.onStartCommand(intent, flags, startId)
        ServiceCompat.startForeground(
            this,
            NOTIFICATION_ID,
            buildNotification(),
            ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE
        )
        // A repeated start does not create a second subscription.
        if (watchJob?.isActive != true) {
            watchJob = lifecycleScope.launch {
                graph.crashSource.tags().collect { graph.handleCrashTag(it) }
            }
        }
        return START_STICKY
    }

    private fun buildNotification(): android.app.Notification {
        getSystemService(NotificationManager::class.java).createNotificationChannel(
            NotificationChannel(CHANNEL_ID, getString(R.string.channel_name), NotificationManager.IMPORTANCE_LOW)
        )
        return NotificationCompat.Builder(this, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.stat_sys_warning)
            .setContentTitle(getString(R.string.notification_title))
            .setOngoing(true)
            .build()
    }

    companion object {
        private const val CHANNEL_ID = "watch"
        private const val NOTIFICATION_ID = 1

        fun start(context: Context) =
            ContextCompat.startForegroundService(context, Intent(context, CrashWatchService::class.java))

        fun stop(context: Context) {
            context.stopService(Intent(context, CrashWatchService::class.java))
        }
    }
}
