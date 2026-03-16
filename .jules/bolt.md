## 2025-05-14 - Replace blocking I/O with tokio::fs in async functions
**Learning:** Direct use of blocking `std::fs` APIs inside an `async` function is an anti-pattern in Rust as it blocks the Tokio executor threads, preventing other tasks from making progress.
**Action:** Always use `tokio::fs` (or `spawn_blocking`) for file I/O operations inside `async` functions to maintain executor responsiveness.
