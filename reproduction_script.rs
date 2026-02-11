use phoenix_lib::backup::BackupManager;
use std::path::PathBuf;
use tokio::time::{sleep, Duration, Instant};
use std::io::Write;

#[tokio::main]
async fn main() {
    // 1. Create a large dummy file (e.g., 50MB)
    let file_path = PathBuf::from("test_large_file.img");
    {
        let mut file = std::fs::File::create(&file_path).unwrap();
        let data = vec![0u8; 1024 * 1024]; // 1MB chunks
        for _ in 0..50 {
            file.write_all(&data).unwrap();
        }
    }
    println!("Created 50MB file.");

    // 2. Spawn a "heartbeat" task that prints every 10ms
    let heartbeat = tokio::spawn(async {
        let mut interval = tokio::time::interval(Duration::from_millis(10));
        let mut last_tick = Instant::now();
        let mut max_delay = Duration::from_secs(0);

        // Let it run for a bit
        loop {
            interval.tick().await;
            let now = Instant::now();
            let elapsed = now.duration_since(last_tick);
            if elapsed > Duration::from_millis(20) {
                 // If it took longer than 20ms between ticks, we were blocked!
                 let delay = elapsed - Duration::from_millis(10);
                 if delay > max_delay {
                     max_delay = delay;
                     // println!("Heartbeat delayed by {:?}", delay);
                 }
            }
            last_tick = now;
        }
    });

    // 3. Run verify_backup
    let start = Instant::now();
    let _ = BackupManager::verify_backup(&file_path).await.unwrap();
    let duration = start.elapsed();

    println!("Verification took {:?}", duration);

    // Stop heartbeat (it runs forever, so just kill the process or let it be killed)
    heartbeat.abort();

    // Clean up
    std::fs::remove_file(file_path).unwrap();
}
