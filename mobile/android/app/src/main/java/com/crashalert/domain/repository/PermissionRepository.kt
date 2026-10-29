package com.crashalert.domain.repository

interface PermissionRepository {
    fun canReadLogs(): Boolean
}
