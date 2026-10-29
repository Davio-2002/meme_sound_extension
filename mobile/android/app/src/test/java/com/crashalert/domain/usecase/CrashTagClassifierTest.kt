package com.crashalert.domain.usecase

import com.crashalert.domain.model.CrashType
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class CrashTagClassifierTest {

    @Test
    fun `app crash tags`() {
        assertEquals(CrashType.APP_CRASH, CrashTagClassifier.classify("data_app_crash"))
        assertEquals(CrashType.APP_CRASH, CrashTagClassifier.classify("system_app_crash"))
    }

    @Test
    fun `native crash tags`() {
        assertEquals(CrashType.NATIVE_CRASH, CrashTagClassifier.classify("data_app_native_crash"))
        assertEquals(CrashType.NATIVE_CRASH, CrashTagClassifier.classify("SYSTEM_TOMBSTONE"))
    }

    @Test
    fun `anr tags`() {
        assertEquals(CrashType.ANR, CrashTagClassifier.classify("data_app_anr"))
        assertEquals(CrashType.ANR, CrashTagClassifier.classify("system_app_anr"))
    }

    @Test
    fun `system tags`() {
        assertEquals(CrashType.SYSTEM, CrashTagClassifier.classify("system_server_crash"))
        assertEquals(CrashType.SYSTEM, CrashTagClassifier.classify("system_server_watchdog"))
        assertEquals(CrashType.SYSTEM, CrashTagClassifier.classify("SYSTEM_RESTART"))
    }

    @Test
    fun `unknown or null tag is ignored`() {
        assertNull(CrashTagClassifier.classify("netstats"))
        assertNull(CrashTagClassifier.classify("data_app_wtf"))
        assertNull(CrashTagClassifier.classify(null))
    }
}
