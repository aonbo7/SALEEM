# Contributing to SALEEM (`CONTRIBUTING.md`)

Thank you for your interest in contributing to **SALEEM**!

## 1. Core Engineering Rules

Before submitting a Pull Request, ensure your changes adhere to our non-negotiable architectural principles:

1. **Real Backend Implementation Only:** Every UI control must be backed by verified Rust logic, structured error handling, and automated tests. Do not add placeholder buttons or fake metrics.
2. **Strict Separation of Responsibilities:**
   - **React + TypeScript (`apps/desktop/src`):** Presentation, user workflows, RTL/LTR localization, and Dark/Light theming only.
   - **Rust (`crates/*` and `apps/desktop/src-tauri`):** All filesystem traversal, path validation, cleanup execution, PE/signature inspection, quarantine, and SQLite persistence.
3. **Filesystem Safety:**
   - Always validate paths through `saleem_core::paths` before reading, moving, or deleting.
   - Never target protected Windows paths (`WinSxS`, `DriverStore`, `System32`, `System Volume Information`).
   - Re-verify file identity (size, modification time, and/or SHA-256) immediately before executing cleanup or quarantine actions (TOCTOU protection).
4. **C-ABI & DLL Discipline:**
   - Do not pass Rust-specific types (`String`, `Vec`, `Result`) directly across `extern "C"` DLL boundaries.
   - Wrap all `extern "C"` functions in `std::panic::catch_unwind`.
   - Ensure memory allocated by an engine DLL is freed via its corresponding `*_free_buffer` function.
5. **Safe Testing:**
   - Never run destructive cleanup or quarantine tests against real user directories. Always use temporary test directories (`tempfile::tempdir()`).

---

## 2. Development Workflow

1. Fork and clone the repository.
2. Create a feature branch (`git checkout -b feat/your-feature` or `fix/your-fix`).
3. Make focused changes with accompanying unit and safety tests.
4. Run verification checks before opening a PR:
   ```powershell
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cd apps/desktop
   npm run typecheck
   npm run build
   ```
5. Open a Pull Request describing the problem, architectural approach, and test results.
