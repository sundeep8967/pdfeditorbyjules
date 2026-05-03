# Master Test Plan: Platform Bindings

## Objective
Verify the iOS (Swift) and Android (Kotlin) wrappers safely bridge to the Rust core without JVM/Obj-C crashes.

## 1. JNI Stability (Android)
- **Vector:** Call Rust memory-allocating functions from Kotlin in a tight `while(true)` loop.
- **Pass Condition:** The JVM Garbage Collector successfully coordinates with the Rust `free` functions via `AutoCloseable` or `Cleaner` registries. No Out-Of-Memory (OOM) exceptions.

## 2. UI Thread Blocking
- **Vector:** Render a 300 DPI page on an older Android device.
- **Pass Condition:** Rendering must occur on a background thread. Main thread must not drop frames.
