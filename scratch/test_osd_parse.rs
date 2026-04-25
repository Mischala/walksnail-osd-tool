fn parse_hex(hex: &str) -> Vec<u8> {
    let clean: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    let mut raw = Vec::new();
    let chars: Vec<char> = clean.chars().collect();
    for i in (0..chars.len()).step_by(2) {
        let s = format!("{}{}", chars[i], chars[i+1]);
        raw.push(u8::from_str_radix(&s, 16).unwrap());
    }
    let mut data = Vec::new();
    for (i, &b) in raw.iter().enumerate() {
        if (i + 1) % 3 != 0 {
            data.push(b);
        }
    }
    data
}

fn main() {
    let hex = "0d00ff0000ffd700ff0000ff9801ff0ab6ff030cff0300ff9034ff2e32ff3206ff07b6ff0309ff1900ff7273ff740affb603ff0d03ff009cff3030ff3a30ff3008ffb603ff0e2cff0041ff4352ff4f08ffb603ff0c2cff0004ff2038ff360bffb603ff0d2bff0020ff2030ff2e35ff379aff09b6ff030eff0400ff2020ff2034ff070fffb603ff1015ff004dff4554ff454fff5237ff3520ff5031ff04b6ff030fff1500ff0ab6ff0303ff0300ff7b37ff3a31ff3030ff0ab6ff0301ff0300ff0120ff3130ff4d57ff08b6ff0302ff0300ff012dff3531ff06b6ff030fff2c00ff4149";
    let data = parse_hex(hex);
    
    println!("Total data len: {}", data.len());
    println!("Num commands: {}", data[0]);
    
    for offset in 8..12 {
        println!("\nTrying offset {}:", offset);
        let mut curr = offset;
        let mut count = 0;
        while curr < data.len() {
            let len = data[curr] as usize;
            if len == 0 || curr + len + 1 > data.len() {
                break;
            }
            // Check for b6 03 pattern
            if data[curr+1] == 0xb6 && data[curr+2] == 0x03 {
                print!("Match! ");
            }
            println!("Cmd {} at {}: len {}", count, curr, len);
            curr += len + 1;
            count += 1;
        }
        println!("Total commands found: {}", count);
    }
}
