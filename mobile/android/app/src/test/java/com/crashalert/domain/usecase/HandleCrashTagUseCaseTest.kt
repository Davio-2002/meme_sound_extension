package com.crashalert.domain.usecase

import com.crashalert.domain.model.CrashType
import com.crashalert.domain.repository.AlertPlayer
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class HandleCrashTagUseCaseTest {

    private val played = mutableListOf<CrashType>()
    private val player = object : AlertPlayer {
        override fun play(type: CrashType) {
            played += type
        }
    }
    private var time = 0L
    private val useCase = HandleCrashTagUseCase(player, now = { time }, cooldownMs = 1_000)

    @Test
    fun `known tag plays sound`() {
        assertEquals(CrashType.ANR, useCase("data_app_anr"))
        assertEquals(listOf(CrashType.ANR), played)
    }

    @Test
    fun `unknown tag is silent`() {
        assertNull(useCase("netstats"))
        assertEquals(emptyList<CrashType>(), played)
    }

    @Test
    fun `second crash inside cooldown is silent`() {
        useCase("data_app_crash")
        time = 500
        assertNull(useCase("data_app_crash"))
        assertEquals(1, played.size)
    }

    @Test
    fun `crash after cooldown plays again`() {
        useCase("data_app_crash")
        time = 1_000
        useCase("system_server_crash")
        assertEquals(listOf(CrashType.APP_CRASH, CrashType.SYSTEM), played)
    }

    @Test
    fun `unknown tag does not start cooldown`() {
        useCase("netstats")
        assertEquals(CrashType.APP_CRASH, useCase("data_app_crash"))
    }
}
