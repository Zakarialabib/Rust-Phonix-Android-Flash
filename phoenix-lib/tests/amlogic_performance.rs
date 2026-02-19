use phoenix_lib::error::AppError;
use phoenix_lib::flash_amlogic::{AmlogicImageHeader, AmlogicPartitionEntry};
use std::io::Write;
use std::path::Path;

fn create_dummy_image(path: &Path, size_mb: usize) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    let chunk = vec![0u8; 1024 * 1024];
    for _ in 0..size_mb {
        file.write_all(&chunk)?;
    }
    Ok(())
}

#[test]
fn test_amlogic_extract_performance() {
    let dir = tempfile::tempdir().unwrap();
    let image_path = dir.path().join("dummy_amlogic_image.img");
    let output_dir = dir.path().join("output_extract");
    let size_mb = 100;

    create_dummy_image(&image_path, size_mb).unwrap();

    let mut partitions = Vec::new();
    // Simulate 50 small partitions (1MB each) to emphasize allocation overhead
    for i in 0..50 {
        partitions.push(AmlogicPartitionEntry {
            name: format!("part_{}", i),
            offset: (i * 1024 * 1024) as u64,
            size: 1024 * 1024,
            verify: false,
        });
    }

    let header = AmlogicImageHeader {
        magic: "@AML".to_string(),
        version: 2,
        chip_id: "Benchmark".to_string(),
        partitions,
    };

    let start = std::time::Instant::now();
    header.extract_to(&image_path, &output_dir).unwrap();
    let duration = start.elapsed();

    println!("Extraction took: {:?}", duration);
}
