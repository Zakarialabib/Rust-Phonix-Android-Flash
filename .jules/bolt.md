## 2026-01-22 - Async Blocking in BackupManager
**Learning:** `BackupManager::verify_backup` was performing CPU-intensive SHA256 hashing directly on the async runtime thread, causing ~63ms blocking delays per 1MB chunk. This violates the architectural rule to use `spawn_blocking` for CPU-bound tasks.
**Action:** Always verify if file processing loops in async functions are CPU-bound (hashing, compression, encryption) and move them to `spawn_blocking` with synchronous I/O to maintain responsiveness.

## 2024-05-22 - [Zip Crate Version Anomaly]
**Learning:** The `zip` crate version in this repo is `7.4.0` in `Cargo.toml` and lockfile, which is unexpected given the public crates.io versions. This version uses generic `FileOptions<T>`.
**Action:** When working with `zip` in this repo, check usages of `FileOptions` carefully.

## 2026-02-12 - Parallelizing Flashing I/O
**Learning:** `flash_image_async` was performing sequential read-then-write operations, leaving the I/O bus idle half the time. By using `tokio::spawn` and `mpsc` channels with buffer recycling, I implemented a producer-consumer pipeline that allows concurrent reading and writing. This significantly improves throughput when source and target are on different buses.
**Action:** Look for opportunities to pipeline sequential I/O operations using channels and spawned tasks, ensuring buffer recycling to avoid allocation overhead.

## 2026-02-12 - Streaming Large File Operations
**Learning:** `RkImageHeader::extract_to` and `parse` were reading entire firmware images (often 2GB+) into memory, causing massive memory pressure and potential OOM.
**Action:** Refactored to use `std::fs::File`, `Seek`, `Read::take`, and `std::io::copy` to stream data. Always verify if file operations on potentially large files are buffered or streaming.