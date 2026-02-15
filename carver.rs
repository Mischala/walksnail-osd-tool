use std::io::{Read, Write, Seek, SeekFrom};
use std::fs::File;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        println!("Usage: carver <input> <output> <offset> <length>");
        return Ok(());
    }

    let input_path = &args[1];
    let output_path = &args[2];
    let offset: u64 = args[3].parse().expect("Invalid offset");
    let length: u64 = args[4].parse().expect("Invalid length");

    let mut input = File::open(input_path)?;
    let mut output = File::create(output_path)?;

    input.seek(SeekFrom::Start(offset))?;
    
    let mut buffer = [0u8; 8192];
    let mut remaining = length;

    while remaining >  0 {
        let to_read = std::cmp::min(remaining, buffer.len() as u64);
        let bytes_read = input.read(&mut buffer[..to_read as usize])?;
        if bytes_read == 0 { break; }
        output.write_all(&buffer[..bytes_read])?;
        remaining -= bytes_read as u64;
    }

    println!("Extracted {} bytes to {}", length - remaining, output_path);
    Ok(())
}
