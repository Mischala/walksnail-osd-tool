use std::io::{Read, Write, Seek, SeekFrom, BufRead, BufReader};
use std::fs::{File, create_dir_all};

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 4 {
        println!("Usage: bulk_carver <input_img> <scan_log> <output_dir>");
        return Ok(());
    }

    let input_path = &args[1];
    let log_path = &args[2];
    let output_dir = &args[3];

    create_dir_all(output_dir)?;

    let mut input = File::open(input_path)?;
    let log_file = File::open(log_path)?;
    println!("Starting bulk carving from {} using log {}", input_path, log_path);
    let mut reader = BufReader::new(log_file);
    let mut line_buf = Vec::new();

    let mut count = 0;
    while reader.read_until(b'\n', &mut line_buf)? > 0 {
        let line = String::from_utf8_lossy(&line_buf);
        if line.contains("Found PNG") {
            if let Some(pos) = line.find("0x") {
                let offset_str = line[pos + 2..].split_whitespace().next().unwrap_or("").trim_matches(|c: char| !c.is_alphanumeric());
                if let Ok(offset) = u64::from_str_radix(offset_str, 16) {
                    count += 1;
                    let output_file_path = format!("{}/png_{:03}.png", output_dir, count);
                    let mut output = File::create(&output_file_path)?;

                    input.seek(SeekFrom::Start(offset))?;
                    
                    let mut buffer = [0u8; 8192];
                    let mut remaining = 200000;
                    while remaining > 0 {
                        let to_read = std::cmp::min(remaining, buffer.len());
                        let bytes_read = input.read(&mut buffer[..to_read])?;
                        if bytes_read == 0 { break; }
                        output.write_all(&buffer[..bytes_read])?;
                        remaining -= bytes_read;
                    }
                    println!("SUCCESS: Carved PNG {} at offset 0x{:X} to {}", count, offset, output_file_path);
                }
            }
        }
        line_buf.clear();
    }
    println!("Finished bulk carving. Total PNGs: {}", count);
    Ok(())
}
