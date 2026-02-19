use phoenix_lib::error::AppError;
use phoenix_lib::flash_rockchip::{RkImageEntry, RkImageHeader};
use std::io::Write;
use std::path::Path;

fn create_fake_rkaf(path: &Path, entry_size: usize) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    let mut header = vec![0u8; 0x8c + 0x70];

    // Magic RKAF
    header[0..4].copy_from_slice(b"RKAF");

    // File size (not strictly used by parse_rkaf for anything but logging, but let's be nice)
    let total_size = (0x8c + 0x70 + entry_size) as u32;
    header[4..8].copy_from_slice(&(total_size - 4).to_le_bytes());

    // Count = 1
    header[0x88..0x8c].copy_from_slice(&1u32.to_le_bytes());

    // Entry 1
    let p = 0x8c;
    let name = b"test_entry";
    header[p..p + name.len()].copy_from_slice(name);

    let entry_path = b"test_path.bin";
    header[p + 0x20..p + 0x20 + entry_path.len()].copy_from_slice(entry_path);

    let offset = 0x8c + 0x70;
    header[p + 0x60..p + 0x64].copy_from_slice(&(offset as u32).to_le_bytes());
    header[p + 0x68..p + 0x6c].copy_from_slice(&(entry_size as u32).to_le_bytes());
    header[p + 0x6c..p + 0x70].copy_from_slice(&(entry_size as u32).to_le_bytes());

    file.write_all(&header)?;

    // Write entry data (large)
    // To avoid using too much disk space, we could use seek, but std::fs::read will still read it all.
    // Let's use 50MB.
    let chunk = vec![0u8; 1024 * 1024];
    for _ in 0..entry_size / (1024 * 1024) {
        file.write_all(&chunk)?;
    }
    file.write_all(&vec![0u8; entry_size % (1024 * 1024)])?;

    Ok(())
}

#[test]
fn test_memory_usage_baseline() {
    let dir = tempfile::tempdir().unwrap();
    let image_path = dir.path().join("large_image.img");
    let output_dir = dir.path().join("output_extract");
    let entry_size = 50 * 1024 * 1024; // 50MB

    create_fake_rkaf(&image_path, entry_size).unwrap();

    let header = RkImageHeader::parse(&image_path).unwrap();
    header.extract_to(&image_path, &output_dir).unwrap();
}
