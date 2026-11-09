// Felles kode for komprimering og dekomprimering: filformat, tabeller,
// bit-lesing/skriving og kanoniske huffmankoder.
//
// Filformat (.komp):
//   u64 LE  originalstørrelse
//   bitstrøm (LSB først) med én eller flere blokker. Hver blokk:
//     1 bit    1 hvis dette er siste blokk
//     blokkhode med kodelengder (se write_block_header)
//     huffmankodede LZ77-symboler, avsluttet med END (256)
//
// Bokstav/lengde-alfabetet: 0..=255 er bytes, 256 er END, 257.. er lengdekoder
// for bakoverreferanser (3..=MAX_MATCH). Avstandsalfabetet dekker 1..=WINDOW_SIZE.
// Tabellene følger samme mønster som deflate, bare utvidet til lengre
// treff og et større vindu.

#![allow(dead_code)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const LEN_CODES: usize = 56;
pub const DIST_CODES: usize = 40;
pub const LIT_SYMBOLS: usize = 257 + LEN_CODES;
pub const DIST_SYMBOLS: usize = DIST_CODES;
pub const END: usize = 256;
pub const MAX_BITS: u8 = 15;

// Lengdekoder: 8 koder uten ekstrabits, deretter 4 koder per antall ekstrabits (1..=12).
const fn len_tables() -> ([u32; LEN_CODES], [u8; LEN_CODES]) {
    let mut base = [0u32; LEN_CODES];
    let mut extra = [0u8; LEN_CODES];
    let mut b = 3u32;
    let mut i = 0;
    while i < LEN_CODES {
        let e = if i < 8 { 0 } else { ((i - 8) / 4 + 1) as u8 };
        base[i] = b;
        extra[i] = e;
        b += 1 << e;
        i += 1;
    }
    (base, extra)
}

// Avstandskoder: 4 koder uten ekstrabits, deretter 2 koder per antall ekstrabits (1..=18).
const fn dist_tables() -> ([u32; DIST_CODES], [u8; DIST_CODES]) {
    let mut base = [0u32; DIST_CODES];
    let mut extra = [0u8; DIST_CODES];
    let mut b = 1u32;
    let mut i = 0;
    while i < DIST_CODES {
        let e = if i < 4 { 0 } else { ((i - 4) / 2 + 1) as u8 };
        base[i] = b;
        extra[i] = e;
        b += 1 << e;
        i += 1;
    }
    (base, extra)
}

pub const LEN_BASE: [u32; LEN_CODES] = len_tables().0;
pub const LEN_EXTRA: [u8; LEN_CODES] = len_tables().1;
pub const DIST_BASE: [u32; DIST_CODES] = dist_tables().0;
pub const DIST_EXTRA: [u8; DIST_CODES] = dist_tables().1;

pub const MIN_MATCH: usize = 3;
pub const MAX_MATCH: usize =
    LEN_BASE[LEN_CODES - 1] as usize + (1 << LEN_EXTRA[LEN_CODES - 1]) - 1; // 32770
pub const WINDOW_SIZE: usize =
    DIST_BASE[DIST_CODES - 1] as usize + (1 << DIST_EXTRA[DIST_CODES - 1]) - 1; // 1 MB

// Største indeks i `base` der base[i] <= verdi.
#[inline]
fn find_code(base: &[u32], value: usize) -> usize {
    base.partition_point(|&b| b as usize <= value) - 1
}

// Lengde (3..=MAX_MATCH) -> (symbol 257.., ekstrabits, verdi på ekstrabits)
#[inline]
pub fn length_code(length: usize) -> (usize, u8, u32) {
    let i = find_code(&LEN_BASE, length);
    (257 + i, LEN_EXTRA[i], (length - LEN_BASE[i] as usize) as u32)
}

// Avstand (1..=WINDOW_SIZE) -> (symbol 0.., ekstrabits, verdi på ekstrabits)
#[inline]
pub fn distance_code(distance: usize) -> (usize, u8, u32) {
    let i = find_code(&DIST_BASE, distance);
    (i, DIST_EXTRA[i], (distance - DIST_BASE[i] as usize) as u32)
}

pub struct BitWriter {
    pub bytes: Vec<u8>,
    buffer: u64,
    bit_count: u32,
}

impl BitWriter {
    pub fn new() -> Self {
        BitWriter { bytes: Vec::new(), buffer: 0, bit_count: 0 }
    }

    // Skriver de `count` laveste bitene i `bits` (maks 32), minst signifikante først.
    #[inline]
    pub fn write_bits(&mut self, bits: u32, count: u8) {
        self.buffer |= (bits as u64) << self.bit_count;
        self.bit_count += count as u32;
        while self.bit_count >= 8 {
            self.bytes.push(self.buffer as u8);
            self.buffer >>= 8;
            self.bit_count -= 8;
        }
    }

    // Fyller siste byte med nuller.
    pub fn finish(mut self) -> Vec<u8> {
        if self.bit_count > 0 {
            self.bytes.push(self.buffer as u8);
        }
        self.bytes
    }
}

pub struct BitReader<'a> {
    data: &'a [u8],
    pos: usize,
    bit: u32,
}

impl<'a> BitReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BitReader { data, pos: 0, bit: 0 }
    }

    #[inline]
    pub fn read_bit(&mut self) -> u32 {
        let byte = *self.data.get(self.pos).expect("uventet slutt på komprimert fil");
        let b = (byte >> self.bit) & 1;
        self.bit += 1;
        if self.bit == 8 {
            self.bit = 0;
            self.pos += 1;
        }
        b as u32
    }

    // Leser `count` bits skrevet med write_bits (minst signifikante først).
    #[inline]
    pub fn read_bits(&mut self, count: u8) -> u32 {
        let mut value = 0;
        for i in 0..count {
            value |= self.read_bit() << i;
        }
        value
    }
}

// Huffmankodelengder fra frekvenser, begrenset til MAX_BITS.
pub fn code_lengths(freq: &[u64]) -> Vec<u8> {
    code_lengths_limited(freq, MAX_BITS)
}

// Symboler med frekvens 0 får lengde 0. Er bare ett symbol brukt, får det lengde 1.
pub fn code_lengths_limited(freq: &[u64], max_bits: u8) -> Vec<u8> {
    let mut freq = freq.to_vec();
    loop {
        let lengths = huffman_lengths(&freq);
        if lengths.iter().all(|&l| l <= max_bits) {
            return lengths;
        }
        // For lange koder: flat ut frekvensene og prøv igjen.
        for f in freq.iter_mut().filter(|f| **f > 0) {
            *f = (*f + 1) / 2;
        }
    }
}

fn huffman_lengths(freq: &[u64]) -> Vec<u8> {
    let mut lengths = vec![0u8; freq.len()];
    let used: Vec<usize> = (0..freq.len()).filter(|&s| freq[s] > 0).collect();
    match used.len() {
        0 => return lengths,
        1 => {
            lengths[used[0]] = 1;
            return lengths;
        }
        _ => {}
    }

    // Noder 0..n er blader (symboler), resten er indre noder. parent[i] peker oppover.
    let mut parent: Vec<usize> = vec![usize::MAX; used.len()];
    let mut heap = BinaryHeap::new();
    for (i, &s) in used.iter().enumerate() {
        heap.push(Reverse((freq[s], i)));
    }
    while heap.len() > 1 {
        let Reverse((f1, a)) = heap.pop().unwrap();
        let Reverse((f2, b)) = heap.pop().unwrap();
        let node = parent.len();
        parent.push(usize::MAX);
        parent[a] = node;
        parent[b] = node;
        heap.push(Reverse((f1 + f2, node)));
    }

    for (i, &s) in used.iter().enumerate() {
        let mut depth = 0;
        let mut node = i;
        while parent[node] != usize::MAX {
            node = parent[node];
            depth += 1;
        }
        lengths[s] = depth.min(255) as u8;
    }
    lengths
}

// Kanoniske koder fra kodelengder. Kodene er bit-reversert, så de kan skrives
// direkte med BitWriter::write_bits og leses bit for bit med Decoder.
pub fn canonical_codes(lengths: &[u8]) -> Vec<u32> {
    let mut count = [0u32; MAX_BITS as usize + 1];
    for &l in lengths {
        count[l as usize] += 1;
    }
    count[0] = 0;

    let mut next = [0u32; MAX_BITS as usize + 2];
    let mut code = 0;
    for bits in 1..=MAX_BITS as usize {
        code = (code + count[bits - 1]) << 1;
        next[bits] = code;
    }

    lengths
        .iter()
        .map(|&l| {
            if l == 0 {
                return 0;
            }
            let c = next[l as usize];
            next[l as usize] += 1;
            reverse_bits(c, l)
        })
        .collect()
}

fn reverse_bits(mut code: u32, length: u8) -> u32 {
    let mut rev = 0;
    for _ in 0..length {
        rev = (rev << 1) | (code & 1);
        code >>= 1;
    }
    rev
}

// Kanonisk huffmandekoder: leser én bit om gangen.
pub struct Decoder {
    count: [u32; MAX_BITS as usize + 1],
    symbols: Vec<usize>, // sortert på (lengde, symbol)
}

impl Decoder {
    pub fn new(lengths: &[u8]) -> Self {
        let mut count = [0u32; MAX_BITS as usize + 1];
        for &l in lengths {
            count[l as usize] += 1;
        }
        count[0] = 0;
        let mut symbols = Vec::new();
        for bits in 1..=MAX_BITS {
            for (s, &l) in lengths.iter().enumerate() {
                if l == bits {
                    symbols.push(s);
                }
            }
        }
        Decoder { count, symbols }
    }

    pub fn decode(&self, reader: &mut BitReader) -> usize {
        let mut code: u32 = 0; // koden lest så langt
        let mut first: u32 = 0; // første kode med denne lengden
        let mut index: u32 = 0; // indeks i `symbols` for første kode med denne lengden
        for bits in 1..=MAX_BITS as usize {
            code |= reader.read_bit();
            let count = self.count[bits];
            if code < first + count {
                return self.symbols[(index + code - first) as usize];
            }
            index += count;
            first = (first + count) << 1;
            code <<= 1;
        }
        panic!("ugyldig huffmankode i komprimert fil");
    }
}

// ---------- blokkhode ----------
//
// Kodelengdene (LIT_SYMBOLS + DIST_SYMBOLS stykker) kjørelengdekodes som i deflate:
//   0..=15  en kodelengde
//   16      gjenta forrige lengde 3-6 ganger   (2 ekstrabits)
//   17      3-10 nuller                        (3 ekstrabits)
//   18      11-138 nuller                      (7 ekstrabits)
// Disse 19 symbolene huffmankodes, og deres kodelengder (maks 7) lagres med 3 bit hver.

const CL_SYMBOLS: usize = 19;

fn rle_lengths(lengths: &[u8]) -> Vec<(u8, u8)> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < lengths.len() {
        let l = lengths[i];
        let mut run = 1;
        while i + run < lengths.len() && lengths[i + run] == l {
            run += 1;
        }
        if l == 0 && run >= 3 {
            let r = run.min(138);
            if r >= 11 {
                out.push((18, (r - 11) as u8));
            } else {
                out.push((17, (r - 3) as u8));
            }
            i += r;
        } else if l != 0 && run >= 4 {
            out.push((l, 0));
            let r = (run - 1).min(6);
            out.push((16, (r - 3) as u8));
            i += 1 + r;
        } else {
            out.push((l, 0));
            i += 1;
        }
    }
    out
}

fn cl_extra_bits(sym: u8) -> u8 {
    match sym {
        16 => 2,
        17 => 3,
        18 => 7,
        _ => 0,
    }
}

pub fn write_block_header(writer: &mut BitWriter, lit_len: &[u8], dist_len: &[u8]) {
    let all: Vec<u8> = lit_len.iter().chain(dist_len.iter()).copied().collect();
    let rle = rle_lengths(&all);

    let mut freq = [0u64; CL_SYMBOLS];
    for &(sym, _) in &rle {
        freq[sym as usize] += 1;
    }
    let cl_len = code_lengths_limited(&freq, 7);
    let cl_code = canonical_codes(&cl_len);

    for &l in &cl_len {
        writer.write_bits(l as u32, 3);
    }
    for &(sym, extra) in &rle {
        writer.write_bits(cl_code[sym as usize], cl_len[sym as usize]);
        let bits = cl_extra_bits(sym);
        if bits > 0 {
            writer.write_bits(extra as u32, bits);
        }
    }
}

pub fn read_block_header(reader: &mut BitReader) -> Result<(Vec<u8>, Vec<u8>), String> {
    let cl_len: Vec<u8> = (0..CL_SYMBOLS).map(|_| reader.read_bits(3) as u8).collect();
    let cl_decoder = Decoder::new(&cl_len);

    let total = LIT_SYMBOLS + DIST_SYMBOLS;
    let mut all: Vec<u8> = Vec::with_capacity(total);
    while all.len() < total {
        let sym = cl_decoder.decode(reader);
        match sym {
            0..=15 => all.push(sym as u8),
            16 => {
                let prev = *all.last().ok_or("gjentakelse uten forrige lengde")?;
                let r = 3 + reader.read_bits(2) as usize;
                all.extend(std::iter::repeat(prev).take(r));
            }
            17 => {
                let r = 3 + reader.read_bits(3) as usize;
                all.extend(std::iter::repeat(0).take(r));
            }
            _ => {
                let r = 11 + reader.read_bits(7) as usize;
                all.extend(std::iter::repeat(0).take(r));
            }
        }
    }
    if all.len() != total || all.iter().any(|&l| l > MAX_BITS) {
        return Err("ugyldig blokkhode".to_string());
    }
    let dist = all.split_off(LIT_SYMBOLS);
    Ok((all, dist))
}
