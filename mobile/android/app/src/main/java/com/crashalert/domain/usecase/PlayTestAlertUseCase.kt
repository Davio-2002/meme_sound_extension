package com.crashalert.domain.usecase

import com.crashalert.domain.model.CrashType
import com.crashalert.domain.repository.AlertPlayer

class PlayTestAlertUseCase(private val player: AlertPlayer) {
    operator fun invoke(type: CrashType) = player.play(type)
}
