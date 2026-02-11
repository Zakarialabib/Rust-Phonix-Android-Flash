## 2026-01-22 - Async Blocking in BackupManager
**Learning:** `BackupManager::verify_backup` was performing CPU-intensive SHA256 hashing directly on the async runtime thread, causing ~63ms blocking delays per 1MB chunk. This violates the architectural rule to use `spawn_blocking` for CPU-bound tasks.
**Action:** Always verify if file processing loops in async functions are CPU-bound (hashing, compression, encryption) and move them to `spawn_blocking` with synchronous I/O to maintain responsiveness.
