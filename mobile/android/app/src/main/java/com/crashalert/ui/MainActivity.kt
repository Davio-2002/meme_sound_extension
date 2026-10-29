package com.crashalert.ui

import android.Manifest
import android.os.Build
import android.os.Bundle
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import androidx.lifecycle.viewmodel.initializer
import androidx.lifecycle.viewmodel.viewModelFactory
import com.crashalert.R
import com.crashalert.databinding.ActivityMainBinding
import com.crashalert.domain.model.CrashType
import com.crashalert.domain.model.SetupStatus
import com.crashalert.graph
import com.crashalert.service.CrashWatchService
import kotlinx.coroutines.launch

class MainActivity : AppCompatActivity() {

    private val viewModel: MainViewModel by viewModels {
        viewModelFactory {
            initializer { graph.run { MainViewModel(getSetupStatus, setWatchEnabled, playTestAlert) } }
        }
    }

    private val notificationPermission =
        registerForActivityResult(ActivityResultContracts.RequestPermission()) {}

    private lateinit var binding: ActivityMainBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityMainBinding.inflate(layoutInflater)
        setContentView(binding.root)

        binding.adbCommand.text = getString(R.string.adb_command, packageName)
        binding.watchSwitch.setOnClickListener { viewModel.onToggle(binding.watchSwitch.isChecked) }
        binding.testApp.setOnClickListener { viewModel.onTestSound(CrashType.APP_CRASH) }
        binding.testAnr.setOnClickListener { viewModel.onTestSound(CrashType.ANR) }
        binding.testSystem.setOnClickListener { viewModel.onTestSound(CrashType.SYSTEM) }

        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            notificationPermission.launch(Manifest.permission.POST_NOTIFICATIONS)
        }

        lifecycleScope.launch {
            repeatOnLifecycle(Lifecycle.State.STARTED) {
                viewModel.state.collect(::render)
            }
        }
    }

    override fun onResume() {
        super.onResume()
        viewModel.refresh()
    }

    private fun render(status: SetupStatus) {
        binding.permissionStatus.setText(
            if (status.canReadLogs) R.string.permission_ok else R.string.permission_missing
        )
        binding.watchSwitch.isEnabled = status.canReadLogs
        binding.watchSwitch.isChecked = status.watchEnabled
        if (status.watchEnabled) CrashWatchService.start(this) else CrashWatchService.stop(this)
    }
}
