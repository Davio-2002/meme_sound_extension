package com.crashalert.domain.repository

import kotlinx.coroutines.flow.Flow

// Stream of tags of new entries in the system crash log.
interface CrashEventSource {
    fun tags(): Flow<String>
}
