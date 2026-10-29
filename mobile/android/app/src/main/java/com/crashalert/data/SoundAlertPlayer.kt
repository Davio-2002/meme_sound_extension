package com.crashalert.data

import android.content.Context
import android.media.AudioAttributes
import android.media.AudioManager
import android.media.MediaPlayer
import com.crashalert.R
import com.crashalert.domain.model.CrashType
import com.crashalert.domain.repository.AlertPlayer

// Plays the custom sound for any failure.
// Uses the alarm stream, so it is audible even when the ringer is silent.
class SoundAlertPlayer(private val context: Context) : AlertPlayer {

    private val attributes = AudioAttributes.Builder()
        .setUsage(AudioAttributes.USAGE_ALARM)
        .setContentType(AudioAttributes.CONTENT_TYPE_SONIFICATION)
        .build()

    private var current: MediaPlayer? = null

    override fun play(type: CrashType) {
        // A new failure restarts the sound.
        current?.release()
        current = MediaPlayer.create(
            context,
            R.raw.crash_sound,
            attributes,
            context.getSystemService(AudioManager::class.java).generateAudioSessionId()
        )?.apply {
            setOnCompletionListener {
                it.release()
                if (current === it) current = null
            }
            start()
        }
    }
}
