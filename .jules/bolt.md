## 2024-05-22 - [Zip Crate Version Anomaly]
**Learning:** The `zip` crate version in this repo is `7.4.0` in `Cargo.toml` and lockfile, which is unexpected given the public crates.io versions. This version uses generic `FileOptions<T>`.
**Action:** When working with `zip` in this repo, check usages of `FileOptions` carefully.
