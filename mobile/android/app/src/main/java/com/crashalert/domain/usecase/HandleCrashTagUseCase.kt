package com.crashalert.domain.usecase

import com.crashalert.domain.model.CrashType
import com.crashalert.domain.repository.AlertPlayer

// Decides whether a tag should trigger the sound and plays it.
// The cooldown prevents noise during a burst of failures.
class HandleCrashTagUseCase(
    private val player: AlertPlayer,
    private val now: () -> Long = System::currentTimeMillis,
    // Slightly longer than the sound, so it plays to the end.
    private val cooldownMs: Long = 4_000
) {
    private var lastPlayAt: Long? = null

    operator fun invoke(tag: String): CrashType? {
        val type = CrashTagClassifier.classify(tag) ?: return null
        val time = now()
        val last = lastPlayAt
        if (last != null && time - last < cooldownMs) return null
        lastPlayAt = time
        player.play(type)
        return type
    }
}
