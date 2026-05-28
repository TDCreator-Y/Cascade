# Debug Session: tauri-cargo-missing

Status: OPEN

Symptom:
- Running `npm run tauri dev` fails with:
  - `failed to run 'cargo metadata'`
  - `program not found`

Initial Hypotheses:
1. Rust toolchain is not installed, so `cargo` is unavailable in PATH.
2. Rust is installed, but the current terminal session does not have PATH configured correctly.
3. The project's Tauri CLI or `src-tauri` config points to an invalid Rust workspace path.
4. `package.json` or Tauri config passes a command that assumes Cargo exists before prechecks.
5. A shell/environment mismatch on Windows causes Node to spawn a process without resolving `cargo.exe`.

Planned Evidence Collection:
- Inspect `package.json` and Tauri config.
- Check whether `cargo`, `rustc`, and `tauri` are available in the terminal.
- Verify `src-tauri` workspace files exist and are structurally valid.
- Reproduce the failure and compare it with environment checks.

Evidence Collected:
- `package.json` contains `"tauri": "tauri"` and `@tauri-apps/cli@2.11.2` is installed.
- `src-tauri/Cargo.toml` exists and is structurally valid for a Tauri v2 app.
- `src-tauri/tauri.conf.json` exists and points to a valid frontend dev URL config.
- `vite.config.ts` explicitly sets port `1420`, matching the Tauri config.
- Terminal evidence:
  - `cargo --version` => command not found
  - `rustc --version` => command not found
  - `where.exe cargo` => file not found
  - `where.exe rustc` => file not found
  - `RUSTUP_HOME` and `CARGO_HOME` are unset

Hypothesis Status:
1. Rust toolchain is not installed, so `cargo` is unavailable in PATH. => CONFIRMED
2. Rust is installed, but the current terminal session does not have PATH configured correctly. => NOT SUPPORTED by current evidence
3. The project's Tauri CLI or `src-tauri` config points to an invalid Rust workspace path. => REJECTED
4. `package.json` or Tauri config passes a command that assumes Cargo exists before prechecks. => PARTIALLY TRUE but not root cause
5. A shell/environment mismatch on Windows causes Node to spawn a process without resolving `cargo.exe`. => POSSIBLE, but lower probability than missing toolchain

Current Root Cause:
- The Windows environment running `npm run tauri dev` does not provide the Rust toolchain (`cargo`/`rustc`), so Tauri fails immediately when invoking `cargo metadata`.
