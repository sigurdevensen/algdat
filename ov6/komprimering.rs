mod felles;

use felles::*;
use std::env;
use std::fs;
use std::process;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

const HASH_BITS: u32 = 18;
const HASH_SIZE: usize = 1 << HASH_BITS;
const HASH3_BITS: u32 = 16;
const HASH3_SIZE: usize = 1 << HASH3_BITS;
const NONE: u32 = u32::MAX;
const MAX_CHAIN: usize = 128; // maks antall kandidater vi sjekker per posisjon
const NICE_LEN: usize = 128; // så lange treff tas direkte, uten optimal parsing inni
const MAX_LAZY: usize = 128; // lengre treff enn dette godtas uten lat sjekk
const CHUNK_SIZE: usize = 1 << 20; // omtrent så mange byte per blokk
const PASSES: usize = 2; // antall runder med optimal parsing per blokk
const UNUSED_COST: u32 = 15; // antatt pris i bits for et symbol som ikke har kode ennå

enum Token {
    Literal(u8),
    Match { length: usize, distance: usize },
}

// Hashkjeder for å finne tidligere forekomster av de neste 4 bytene, pluss en tabell
// med siste forekomst av de neste 3 bytene (for treff på lengde 3).
// head[h] er siste posisjon med hash h, prev[pos % WINDOW_SIZE] er forrige med samme hash.
struct MatchFinder<'a> {
    data: &'a [u8],
    head: Vec<u32>,
    prev: Vec<u32>,
    head3: Vec<u32>,
}

impl<'a> MatchFinder<'a> {
    fn new(data: &'a [u8]) -> Self {
        MatchFinder {
            data,
            head: vec![NONE; HASH_SIZE],
            prev: vec![NONE; WINDOW_SIZE],
            head3: vec![NONE; HASH3_SIZE],
        }
    }

    #[inline]
    fn hash3(&self, pos: usize) -> usize {
        let d = self.data;
        let v = d[pos] as u32 | (d[pos + 1] as u32) << 8 | (d[pos + 2] as u32) << 16;
        (v.wrapping_mul(0x9E37_79B1) >> (32 - HASH3_BITS)) as usize
    }

    #[inline]
    fn hash4(&self, pos: usize) -> usize {
        let v = u32::from_le_bytes(self.data[pos..pos + 4].try_into().unwrap());
        (v.wrapping_mul(0x9E37_79B1) >> (32 - HASH_BITS)) as usize
    }

    #[inline]
    fn insert(&mut self, pos: usize) {
        if pos + 4 <= self.data.len() {
            let h = self.hash4(pos);
            self.prev[pos & (WINDOW_SIZE - 1)] = self.head[h];
            self.head[h] = pos as u32;
        }
        if pos + MIN_MATCH <= self.data.len() {
            let h = self.hash3(pos);
            self.head3[h] = pos as u32;
        }
    }

    #[inline]
    fn read_u32(&self, pos: usize) -> u32 {
        debug_assert!(pos + 4 <= self.data.len());
        // SAFETY: kallerne sørger for at pos + 4 <= data.len()
        unsafe { (self.data.as_ptr().add(pos) as *const u32).read_unaligned() }
    }

    #[inline]
    fn read_u64(&self, pos: usize) -> u64 {
        debug_assert!(pos + 8 <= self.data.len());
        // SAFETY: kallerne sørger for at pos + 8 <= data.len()
        u64::from_le(unsafe { (self.data.as_ptr().add(pos) as *const u64).read_unaligned() })
    }

    // Antall like bytes fra posisjon a og b, maks max_len. Sammenligner 8 bytes om gangen.
    #[inline]
    fn match_length(&self, a: usize, b: usize, max_len: usize) -> usize {
        let d = self.data;
        let mut len = 0;
        while len + 8 <= max_len {
            let diff = self.read_u64(a + len) ^ self.read_u64(b + len);
            if diff != 0 {
                return len + (diff.trailing_zeros() / 8) as usize;
            }
            len += 8;
        }
        while len < max_len && d[a + len] == d[b + len] {
            len += 1;
        }
        len
    }

    // Alle treff for posisjon `pos` (som ikke er satt inn ennå) som er lengre enn
    // de forrige: (lengde, avstand) med økende lengde og nærmeste avstand for hver lengde.
    fn find_matches(&self, pos: usize, out: &mut Vec<(u32, u32)>) {
        out.clear();
        let data = self.data;
        if pos + MIN_MATCH > data.len() {
            return;
        }
        let max_len = MAX_MATCH.min(data.len() - pos);
        let mut best_len = MIN_MATCH - 1;

        // Nærmeste treff på 3 bytes
        let c = self.head3[self.hash3(pos)];
        if c != NONE && pos - c as usize <= WINDOW_SIZE {
            let len = self.match_length(c as usize, pos, max_len);
            if len >= MIN_MATCH {
                best_len = len;
                out.push((len as u32, (pos - c as usize) as u32));
                if len >= NICE_LEN || len == max_len {
                    return;
                }
            }
        }
        if pos + 4 > data.len() {
            return;
        }

        let mut candidate = self.head[self.hash4(pos)];
        let mut depth = 0;
        while candidate != NONE && depth < MAX_CHAIN {
            let c = candidate as usize;
            let distance = pos - c;
            if distance > WINDOW_SIZE {
                break;
            }
            // Rask sjekk: de 4 bytene som slutter på posisjon best_len må være like
            let off = best_len.max(3) - 3;
            if self.read_u32(c + off) == self.read_u32(pos + off) {
                let len = self.match_length(c, pos, max_len);
                if len > best_len {
                    best_len = len;
                    out.push((len as u32, distance as u32));
                    if len >= NICE_LEN || len == max_len {
                        break;
                    }
                }
            }
            let next = self.prev[c & (WINDOW_SIZE - 1)];
            // Gamle oppføringer i ringbufferet kan peke fremover; da stopper kjeden
            if next == NONE || next >= candidate {
                break;
            }
            candidate = next;
            depth += 1;
        }
    }
}

// Treffene for hver posisjon i en blokk: treff for posisjon i ligger i
// matches[offsets[i]..offsets[i + 1]].
struct MatchCache {
    offsets: Vec<u32>,
    matches: Vec<(u32, u32)>,
}

// Søker opp treff for alle posisjoner i start..end og setter dem inn i hashkjedene.
fn collect_matches(
    finder: &mut MatchFinder,
    start: usize,
    end: usize,
    cache: &mut MatchCache,
    scratch: &mut Vec<(u32, u32)>,
) {
    cache.offsets.clear();
    cache.matches.clear();
    let mut skip_until = start;
    for pos in start..end {
        cache.offsets.push(cache.matches.len() as u32);
        if pos >= skip_until {
            finder.find_matches(pos, scratch);
            cache.matches.extend_from_slice(scratch);
            if let Some(&(len, _)) = scratch.last() {
                if len as usize >= NICE_LEN {
                    // Langt treff: tas direkte, så posisjonene inni trenger ikke søk
                    skip_until = pos + len as usize;
                }
            }
        }
        finder.insert(pos);
    }
    cache.offsets.push(cache.matches.len() as u32);
}

// Pris i bits for hvert symbol, ut fra kodelengder.
struct Costs {
    literal: [u32; 256],
    length: Vec<u32>,               // indeks er treff-lengden, inkludert ekstrabits
    distance: [u32; DIST_SYMBOLS], // inkludert ekstrabits
}

fn bits(len: u8) -> u32 {
    if len == 0 { UNUSED_COST } else { len as u32 }
}

impl Costs {
    fn new(lit_len: &[u8], dist_len: &[u8]) -> Self {
        let mut literal = [0u32; 256];
        for b in 0..256 {
            literal[b] = bits(lit_len[b]);
        }
        let mut length = vec![0u32; MAX_MATCH + 1];
        for l in MIN_MATCH..=MAX_MATCH {
            let (symbol, extra_bits, _) = length_code(l);
            length[l] = bits(lit_len[symbol]) + extra_bits as u32;
        }
        let mut distance = [0u32; DIST_SYMBOLS];
        for s in 0..DIST_SYMBOLS {
            distance[s] = bits(dist_len[s]) + DIST_EXTRA[s] as u32;
        }
        Costs { literal, length, distance }
    }

    #[inline]
    fn distance_cost(&self, distance: usize) -> u32 {
        self.distance[distance_code(distance).0]
    }
}

// Lat parsing ut fra de lagrede treffene, brukt for å få startpriser til første blokk.
fn lazy_parse(block: &[u8], cache: &MatchCache, tokens: &mut Vec<Token>) {
    tokens.clear();
    let n = block.len();
    let longest = |i: usize| -> (usize, usize) {
        if i >= n {
            return (0, 0);
        }
        let entries = &cache.matches[cache.offsets[i] as usize..cache.offsets[i + 1] as usize];
        match entries.last() {
            Some(&(len, dist)) => ((len as usize).min(n - i), dist as usize),
            None => (0, 0),
        }
    };
    let mut pos = 0;
    while pos < n {
        let (len, dist) = longest(pos);
        if len < MIN_MATCH || (len < MAX_LAZY && longest(pos + 1).0 > len) {
            tokens.push(Token::Literal(block[pos]));
            pos += 1;
        } else {
            tokens.push(Token::Match { length: len, distance: dist });
            pos += len;
        }
    }
}

// Optimal parsing: korteste vei gjennom blokken der prisen er antall bits.
// cost[i] er billigste pris for de i første bytene, back[i] er siste token dit (lengde, avstand).
fn optimal_parse(
    block: &[u8],
    cache: &MatchCache,
    costs: &Costs,
    cost: &mut Vec<u32>,
    back: &mut Vec<(u32, u32)>,
    tokens: &mut Vec<Token>,
) {
    let n = block.len();
    cost.clear();
    cost.resize(n + 1, u32::MAX);
    back.clear();
    back.resize(n + 1, (0, 0));
    cost[0] = 0;

    for i in 0..n {
        let c = cost[i];
        let lc = c + costs.literal[block[i] as usize];
        if lc < cost[i + 1] {
            cost[i + 1] = lc;
            back[i + 1] = (1, 0);
        }

        let entries = &cache.matches[cache.offsets[i] as usize..cache.offsets[i + 1] as usize];
        let Some(&(last_len, last_dist)) = entries.last() else { continue };
        let remaining = n - i;

        if last_len as usize >= NICE_LEN && last_len as usize <= remaining {
            // Langt treff: prøv bare full lengde
            let l = last_len as usize;
            let t = c + costs.length[l] + costs.distance_cost(last_dist as usize);
            if t < cost[i + l] {
                cost[i + l] = t;
                back[i + l] = (last_len, last_dist);
            }
            continue;
        }

        // Hver avstand dekker lengdene mellom forrige treff sin lengde og sin egen
        let mut prev = MIN_MATCH - 1;
        for &(len, dist) in entries {
            let len = (len as usize).min(remaining);
            if len <= prev {
                break;
            }
            let base = c + costs.distance_cost(dist as usize);
            for l in prev + 1..=len {
                let t = base + costs.length[l];
                if t < cost[i + l] {
                    cost[i + l] = t;
                    back[i + l] = (l as u32, dist);
                }
            }
            prev = len;
        }
    }

    // Gå baklengs fra slutten for å finne tokenene
    tokens.clear();
    let mut i = n;
    while i > 0 {
        let (len, dist) = back[i];
        let len = len as usize;
        if len == 1 {
            tokens.push(Token::Literal(block[i - 1]));
        } else {
            tokens.push(Token::Match { length: len, distance: dist as usize });
        }
        i -= len;
    }
    tokens.reverse();
}

fn count_frequencies(tokens: &[Token]) -> (Vec<u64>, Vec<u64>) {
    let mut lit_freq = vec![0u64; LIT_SYMBOLS];
    let mut dist_freq = vec![0u64; DIST_SYMBOLS];
    for token in tokens {
        match *token {
            Token::Literal(b) => lit_freq[b as usize] += 1,
            Token::Match { length, distance } => {
                lit_freq[length_code(length).0] += 1;
                dist_freq[distance_code(distance).0] += 1;
            }
        }
    }
    lit_freq[END] = 1;
    (lit_freq, dist_freq)
}

fn block_lengths(tokens: &[Token]) -> (Vec<u8>, Vec<u8>) {
    let (lit_freq, dist_freq) = count_frequencies(tokens);
    (code_lengths(&lit_freq), code_lengths(&dist_freq))
}

// Skriver én blokk: sistebit, kodelengder, tokenene og END.
fn write_block(writer: &mut BitWriter, tokens: &[Token], lit_len: &[u8], dist_len: &[u8], last: bool) {
    let lit_code = canonical_codes(lit_len);
    let dist_code = canonical_codes(dist_len);

    writer.write_bits(last as u32, 1);
    write_block_header(writer, lit_len, dist_len);
    for token in tokens {
        match *token {
            Token::Literal(b) => {
                let b = b as usize;
                writer.write_bits(lit_code[b], lit_len[b]);
            }
            Token::Match { length, distance } => {
                let (symbol, extra_bits, extra) = length_code(length);
                writer.write_bits(lit_code[symbol], lit_len[symbol]);
                if extra_bits > 0 {
                    writer.write_bits(extra, extra_bits);
                }
                let (symbol, extra_bits, extra) = distance_code(distance);
                writer.write_bits(dist_code[symbol], dist_len[symbol]);
                if extra_bits > 0 {
                    writer.write_bits(extra, extra_bits);
                }
            }
        }
    }
    writer.write_bits(lit_code[END], lit_len[END]);
}

struct Stats {
    literals: usize,
    matches: usize,
    blocks: usize,
}

// Resultatet for én blokk: tokenene og kodelengdene de skal skrives med.
struct Block {
    tokens: Vec<Token>,
    lit_len: Vec<u8>,
    dist_len: Vec<u8>,
}

// Komprimerer data[start..end] til én blokk. Hver blokk har sin egen MatchFinder,
// som først fylles med vinduet foran blokken, så blokkene kan lages i parallell.
fn compress_block(data: &[u8], start: usize, end: usize) -> Block {
    let mut finder = MatchFinder::new(data);
    for pos in start.saturating_sub(WINDOW_SIZE)..start {
        finder.insert(pos);
    }
    let mut cache = MatchCache { offsets: Vec::new(), matches: Vec::new() };
    let mut scratch = Vec::new();
    collect_matches(&mut finder, start, end, &mut cache, &mut scratch);
    drop(finder);

    let block = &data[start..end];
    let mut tokens = Vec::new();
    let mut cost = Vec::new();
    let mut back = Vec::new();

    // Startpriser fra lat parsing, deretter optimal parsing med prisene fra forrige runde
    lazy_parse(block, &cache, &mut tokens);
    let (mut lit_len, mut dist_len) = block_lengths(&tokens);
    for _ in 0..PASSES {
        let costs = Costs::new(&lit_len, &dist_len);
        optimal_parse(block, &cache, &costs, &mut cost, &mut back, &mut tokens);
        (lit_len, dist_len) = block_lengths(&tokens);
    }
    Block { tokens, lit_len, dist_len }
}

fn compress(data: &[u8]) -> (Vec<u8>, Stats) {
    let mut stats = Stats { literals: 0, matches: 0, blocks: 0 };
    let mut writer = BitWriter::new();

    if data.is_empty() {
        // Tom fil: én siste blokk med bare END
        let (lit_len, dist_len) = block_lengths(&[]);
        write_block(&mut writer, &[], &lit_len, &dist_len, true);
        stats.blocks = 1;
    }

    // Trådene henter neste ledige blokk fra `next_block` og sender resultatet tilbake.
    // Hovedtråden skriver blokkene i rekkefølge etter hvert som de blir ferdige.
    let threads = thread::available_parallelism().map_or(1, |n| n.get());
    let block_count = (data.len() + CHUNK_SIZE - 1) / CHUNK_SIZE;
    let next_block = AtomicUsize::new(0);
    let (sender, receiver) = mpsc::channel::<(usize, Block)>();
    thread::scope(|scope| {
        for _ in 0..threads.min(block_count) {
            let sender = sender.clone();
            let next_block = &next_block;
            scope.spawn(move || loop {
                let b = next_block.fetch_add(1, Ordering::Relaxed);
                if b >= block_count {
                    break;
                }
                let start = b * CHUNK_SIZE;
                let end = (start + CHUNK_SIZE).min(data.len());
                sender.send((b, compress_block(data, start, end))).unwrap();
            });
        }
        drop(sender);

        let mut waiting: BTreeMap<usize, Block> = BTreeMap::new();
        for b in 0..block_count {
            let block = loop {
                if let Some(block) = waiting.remove(&b) {
                    break block;
                }
                let (i, block) = receiver.recv().unwrap();
                waiting.insert(i, block);
            };
            let last = b + 1 == block_count;
            write_block(&mut writer, &block.tokens, &block.lit_len, &block.dist_len, last);
            for token in &block.tokens {
                match token {
                    Token::Literal(_) => stats.literals += 1,
                    Token::Match { .. } => stats.matches += 1,
                }
            }
            stats.blocks += 1;
        }
    });

    let mut out = Vec::with_capacity(8 + data.len() / 2);
    out.extend_from_slice(&(data.len() as u64).to_le_bytes());
    out.extend_from_slice(&writer.finish());
    (out, stats)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Bruk: komprimering <innfil> [utfil]");
        eprintln!("Standard utfil er <innfil>.komp");
        process::exit(1);
    }
    let infile = &args[1];
    let outfile = match args.get(2) {
        Some(f) => f.clone(),
        None => format!("{}.komp", infile),
    };

    let data = fs::read(infile).unwrap_or_else(|e| {
        eprintln!("Kunne ikke lese {}: {}", infile, e);
        process::exit(1);
    });
    if data.len() > u32::MAX as usize {
        eprintln!("Filen er for stor (maks 4 GB)");
        process::exit(1);
    }

    let start = Instant::now();
    let (compressed, stats) = compress(&data);
    let elapsed = start.elapsed();

    fs::write(&outfile, &compressed).unwrap_or_else(|e| {
        eprintln!("Kunne ikke skrive {}: {}", outfile, e);
        process::exit(1);
    });

    println!("Innfil: {} ({} byte)", infile, data.len());
    println!("Utfil: {} ({} byte)", outfile, compressed.len());
    if !data.is_empty() {
        let percent = 100.0 * compressed.len() as f64 / data.len() as f64;
        println!("Kompresjon: komprimert er {:.1}% av originalen", percent);
    }
    println!("Bokstaver: {}, referanser: {}, blokker: {}", stats.literals, stats.matches, stats.blocks);
    println!("Tid: {:.2} s", elapsed.as_secs_f64());
}
