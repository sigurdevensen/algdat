mod felles;

use felles::*;
use std::env;
use std::fs;
use std::process;

fn default_output_name(input: &str) -> String {
    let base = input.strip_suffix(".komp").unwrap_or(input);
    format!("{}.dekomp", base)
}

fn fail(message: &str) -> ! {
    eprintln!("{}", message);
    process::exit(1);
}

fn decompress(data: &[u8]) -> Vec<u8> {
    if data.len() < 8 {
        fail("Feil: filen er for kort til å være en komprimert fil");
    }

    let mut size_bytes = [0u8; 8];
    size_bytes.copy_from_slice(&data[0..8]);
    let original_size = u64::from_le_bytes(size_bytes) as usize;

    let mut reader = BitReader::new(&data[8..]);
    let mut out: Vec<u8> = Vec::with_capacity(original_size);

    loop {
        let last = reader.read_bit() == 1;
        let (lit_lengths, dist_lengths) = read_block_header(&mut reader)
            .unwrap_or_else(|e| fail(&format!("Feil: ugyldig blokkhode ({}), filen er korrupt", e)));
        let lit_decoder = Decoder::new(&lit_lengths);
        let dist_decoder = Decoder::new(&dist_lengths);

        loop {
            let sym = lit_decoder.decode(&mut reader);
            if sym < 256 {
                out.push(sym as u8);
            } else if sym == END {
                break;
            } else if sym < 257 + LEN_CODES {
                let i = sym - 257;
                let length = LEN_BASE[i] as usize + reader.read_bits(LEN_EXTRA[i]) as usize;

                let d = dist_decoder.decode(&mut reader);
                if d >= DIST_CODES {
                    fail("Feil: ugyldig avstandssymbol, filen er korrupt");
                }
                let distance = DIST_BASE[d] as usize + reader.read_bits(DIST_EXTRA[d]) as usize;
                if distance < 1 || distance > out.len() {
                    fail("Feil: ugyldig avstand, filen er korrupt");
                }

                let start = out.len() - distance;
                if distance >= length {
                    out.extend_from_within(start..start + length);
                } else {
                    // Byte for byte, så overlappende kopier fungerer
                    for k in 0..length {
                        let byte = out[start + k];
                        out.push(byte);
                    }
                }
            } else {
                fail("Feil: ugyldig symbol, filen er korrupt");
            }
        }

        if last {
            break;
        }
    }

    if out.len() != original_size {
        fail(&format!(
            "Feil: forventet {} byte, men fikk {} byte",
            original_size,
            out.len()
        ));
    }
    out
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        fail("Bruk: dekomprimering <innfil.komp> [utfil]");
    }

    let input = &args[1];
    let output = match args.get(2) {
        Some(name) => name.clone(),
        None => default_output_name(input),
    };

    let data = fs::read(input).unwrap_or_else(|_| fail(&format!("Finner ikke filen {}", input)));
    let out = decompress(&data);

    fs::write(&output, &out).unwrap_or_else(|_| fail(&format!("Kunne ikke skrive til {}", output)));

    println!("Dekomprimerte {} ({} byte) til {} ({} byte)", input, data.len(), output, out.len());
}
