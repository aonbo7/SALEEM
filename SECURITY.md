# Security Policy (`SECURITY.md`)

## 1. Security Philosophy & Non-Negotiable Boundaries

**SALEEM** is built around user safety, transparency, and least privilege:

1. **No Silent or Unconfirmed Deletions:** SALEEM never deletes or quarantines user files without an explicit preview and user confirmation.
2. **Respect for Windows OS Protection:** SALEEM never attempts to bypass Windows ACLs (`TakeOwnership` / `icacls`), terminate system processes to unlock files, or delete protected system stores (`C:\Windows\WinSxS`, `DriverStore`, `System32`, `System Volume Information`, or System Restore points).
3. **Least Privilege:** SALEEM runs as a standard user by default and does not request Administrator elevation at startup.
4. **No Automatic Cloud Uploads:** All disk analysis, cleanup evaluation, and `SALEEM Sentinel` security scans run **100% locally** by default. User files and paths are never uploaded to external servers automatically.
5. **Non-Execution of Analyzed Files:** `SALEEM Sentinel` performs purely static inspection (SHA-256, content magic bytes, PE headers, Authenticode signatures, and declarative rules). Analyzed files are never executed or loaded as executable code.
6. **Secure Native Engine Loading:** Engine DLLs (`SALEEM.DiskEngine.dll`, `SALEEM.CleanupEngine.dll`, `SALEEM.Sentinel.dll`) are loaded exclusively from verified application paths using `LoadLibraryExW` with restricted search flags and ABI version verification to prevent DLL Search Order Hijacking.

---

## 2. Explicit Scope & Limitations of SALEEM Sentinel

`SALEEM Sentinel` is a **custom static file analysis and risk-assessment engine**, **NOT** a full real-time antivirus replacement:
- It does **not** include a kernel-mode driver, real-time memory scanner, or behavioral process interceptor.
- It cannot detect every zero-day or polymorphic threat.
- An unsigned file is **not** automatically classified as malware, and a signed file is **not** guaranteed to be benign.
- Findings always separate **Risk Level** (`Informational`, `Low`, `Medium`, `High`, `Critical`) from **Confidence** (`Low`, `Medium`, `High`) and provide human-readable evidence.

---

## 3. Supported Versions

| Version | Supported |
|---|---|
| `0.1.x` (Active Development) | Yes |

---

## 4. Reporting a Vulnerability

If you discover a security vulnerability in **SALEEM** (such as path traversal, symlink/junction escape, TOCTOU race condition in cleanup/quarantine, PE parser memory safety issue, IPC privilege escalation, or DLL hijacking vector):

1. **Do NOT open a public GitHub issue** with exploit details until maintainers have reviewed and patched the issue.
2. Report the vulnerability privately via GitHub Security Advisories (`Security -> Report a vulnerability`) in the official `SALEEM` repository.
3. Include:
   - Affected component (`SALEEM Core`, `SALEEM Disk Engine`, `SALEEM Cleanup Engine`, `SALEEM Sentinel`, `SALEEM Quarantine`, or `Desktop UI/IPC`).
   - Step-by-step reproduction instructions or proof-of-concept test fixture.
   - Expected vs. actual behavior and potential impact.
4. Maintainers will acknowledge receipt within **72 hours** and work on a verified fix and regression test prior to public disclosure.
