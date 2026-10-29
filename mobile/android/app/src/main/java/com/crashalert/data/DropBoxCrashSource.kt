package com.crashalert.data

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.DropBoxManager
import androidx.core.content.ContextCompat
import com.crashalert.domain.repository.CrashEventSource
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow

// The system writes every crash/ANR to DropBox and sends a broadcast.
// Only apps with READ_LOGS (and READ_DROPBOX_DATA on Android 15+) receive it.
class DropBoxCrashSource(private val context: Context) : CrashEventSource {

    override fun tags(): Flow<String> = callbackFlow {
        val receiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context, intent: Intent) {
                intent.getStringExtra(DropBoxManager.EXTRA_TAG)?.let { trySend(it) }
            }
        }
        ContextCompat.registerReceiver(
            context,
            receiver,
            IntentFilter(DropBoxManager.ACTION_DROPBOX_ENTRY_ADDED),
            ContextCompat.RECEIVER_NOT_EXPORTED
        )
        awaitClose { context.unregisterReceiver(receiver) }
    }
}
