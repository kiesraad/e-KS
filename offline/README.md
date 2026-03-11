# e-KS Desktop (Tauri) - Proof of Concept

This is a **desktop version** of the e-KS application.

It is a **proof-of-concept** and **not production-ready**.

## Security Work Needed
- Tighten security by encrypting storage/state using the OS secret vault.
- Protect communication using the Tauri protocol instead of HTTP.

## Running the Desktop App
1. Install prerequisites (see the official Tauri guide):
   ```
   https://v2.tauri.app/start/prerequisites/
   ```
2. From the `offline/` directory, build and run the app with Cargo:
   ```bash
   cargo tauri dev
   ```
3. Note that downloads do not work yet, and the app is not fully functional. This is just a starting point for development.
