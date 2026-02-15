use std::io::{Read, Seek, SeekFrom};
use std::fs::File;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Usage: scanner <file>");
        return Ok(());
    }

    let file_path = &args[1];
    let mut file = File::open(file_path)?;
    let mut buffer = [0u8; 4096];

    println!("Scanning {}...", file_path);

    let signatures: &[(&str, &[u8])] = &[
        ("OKLI", b"OKLI"),
        ("PNG", b"\x89PNG\r\n\x1a\n"),
        ("SquashFS", b"hsqs"),
        ("SquashFS-be", b"sqsh"),
        ("EXT4", b"\x53\xEF"), // Offset 0x438 in superblock
        ("U-Boot", b"\x27\x05\x19\x56"),
        ("GZIP", b"\x1F\x8B"),
        ("LZMA", b"\x5D\x00\x00"),
    ];

    let mut offset = 0u64;
    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 { break; }

        for i in 0..bytes_read {
            for (name, sig) in signatures {
                if i + sig.len() <= bytes_read {
                    if &buffer[i..i+sig.len()] == *sig {
                        // For GZIP, we can estimate length from headers or just scan for signatures
                        println!("Found {} at offset 0x{:X}", name, offset + i as u64);
                    }
                }
            }
        }
        offset += bytes_read as u64;
        
        // Overlap buffer to catch signatures across boundaries
        if offset > 16 {
             file.seek(SeekFrom::Start(offset - 16))?;
             offset -= 16;
        }
    }

    Ok(())
}
