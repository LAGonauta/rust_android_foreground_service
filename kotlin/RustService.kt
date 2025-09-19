package com.test.foregroundservice

import android.app.Service
import android.content.Intent
import android.os.IBinder
import android.util.Log

class RustService : Service() {
    /**
     * Load our Rust library once the service is created.
     * The library name must match what's in your Cargo.toml (e.g., name = "my_rust_app")
     * but without the "lib" prefix and ".so" suffix.
     */
    init {
        System.loadLibrary("template")
    }

    /**
     * Declare the native functions that will be implemented in Rust.
     * These are the bridges from Kotlin to Rust.
     */
    private external fun startRustLogic(service: Service)
    private external fun stopRustLogic()

    override fun onCreate() {
        super.onCreate()
        Log.d("RustService", "Service onCreate: Loading Rust logic.")
        // Call the native Rust function, passing a reference to this service instance.
        startRustLogic(this)
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        // We handle logic in onCreate triggered by startRustLogic.
        // START_STICKY tells the OS to recreate the service if it's killed.
        return START_STICKY
    }

    override fun onDestroy() {
        Log.d("RustService", "Service onDestroy: Stopping Rust logic.")
        stopRustLogic()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? {
        // We don't provide binding, so return null
        return null
    }
}