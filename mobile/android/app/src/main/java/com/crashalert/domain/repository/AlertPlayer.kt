package com.crashalert.domain.repository

import com.crashalert.domain.model.CrashType

// Plays the alert sound for a crash type.
interface AlertPlayer {
    fun play(type: CrashType)
}
