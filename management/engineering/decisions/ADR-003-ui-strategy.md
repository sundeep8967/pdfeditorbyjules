# ADR 003: UI and Frontend Strategy

## Status
Accepted

## Context
Our goal is to release iOS and Android apps with PDF editing capabilities.

## Decision
For now, we are building zero UI. Our entire focus is building the headless Rust SDK. The Rust engine will expose a Foreign Function Interface (FFI).
When the SDK reaches MVP status, we will evaluate whether to build the wrappers in Native UI (Swift/Kotlin) or cross-platform UI (React Native / Flutter).

## Consequences
- No UI developers are needed immediately.
- We must build rigorous unit tests in Rust to prove the rendering and editing logic works before we have an app to see it in.
