package com.crashalert.data

import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import com.crashalert.domain.repository.PermissionRepository

class AndroidPermissionRepository(private val context: Context) : PermissionRepository {

    // Android 15+ also requires READ_DROPBOX_DATA for crash broadcasts.
    private val required = buildList {
        add("android.permission.READ_LOGS")
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.VANILLA_ICE_CREAM) {
            add("android.permission.READ_DROPBOX_DATA")
        }
    }

    override fun canReadLogs() = required.all {
        context.checkSelfPermission(it) == PackageManager.PERMISSION_GRANTED
    }
}
