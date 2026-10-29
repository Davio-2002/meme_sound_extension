package com.crashalert.domain.model

// What is ready for monitoring.
data class SetupStatus(
    val canReadLogs: Boolean,
    val watchEnabled: Boolean
)
