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

## 2026-02-12 - [Buffered Writing in Asset Downloads]
**Learning:** `download_file` in `phoenix-lib/src/assets.rs` was using unbuffered `tokio::fs::File::write_all` inside a loop receiving network chunks. This resulted in excessive syscalls for small chunks. Benchmark showed ~2x speedup by wrapping `File` in `tokio::io::BufWriter`.
**Action:** Always wrap file writers in `BufWriter` when writing data in small or variable chunks, especially in async contexts where syscall overhead adds up.

## 2026-03-04 - [Zip Archive Extraction Overhead]
**Learning:** Extracting zip archives with many small files was slow due to `std::io::copy` combined with a `BufWriter` being allocated for every single file. Even without `BufWriter`, `std::io::copy` allocates an 8KB buffer internally. By hoisting a 1MB `Vec<u8>` buffer allocation outside the extraction loop and using manual `read`/`write_all`, I halved the extraction time by removing thousands of unnecessary allocations.
**Action:** When extracting archives or processing many small files in a loop, always allocate your I/O buffer once outside the loop and manually read/write to reuse it, instead of using `std::io::copy` or per-file `BufWriter`s.
