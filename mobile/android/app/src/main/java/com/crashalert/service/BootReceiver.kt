package com.crashalert.service

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import com.crashalert.graph

// Restarts monitoring after reboot if it was enabled.
class BootReceiver : BroadcastReceiver() {
    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action != Intent.ACTION_BOOT_COMPLETED) return
        val status = context.graph.getSetupStatus()
        if (status.watchEnabled && status.canReadLogs) CrashWatchService.start(context)
    }
}
