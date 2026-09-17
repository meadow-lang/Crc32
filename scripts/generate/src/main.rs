//! Writes `src/Cases.mw` for crc32.
//!
//! ```text
//! cargo run --release -- <package root>
//! ```
//!
//! Bytes, with the checksums crc32fast gives them: hashed whole, in pieces,
//! from an initial state, and combined. The library is ported by hand into
//! `src/`, and the crate's source is fingerprinted.

use crc32fast::Hasher;
use std::fmt::Write as _;
use std::path::PathBuf;

/// The crate version pinned in `Cargo.toml`.
const UPSTREAM_VERSION: &str = "1.5.2";

/// The fingerprint of the crate's source, which `src/` ports.
const SOURCES: u64 = 0xbf4d_a85f_c735_cff7;

fn main() {
    let root = PathBuf::from(std::env::args().nth(1).unwrap_or_else(|| "../..".into()));

    let print = fingerprint(include_str!(concat!(env!("OUT_DIR"), "/sources.rs.txt")));
    if print != SOURCES {
        eprintln!(
            "error: crc32fast is not the version src/ ports.\n\
             Compare its source in {} with the previous version, carry any change\n\
             into src/, then set SOURCES in scripts/generate/src/main.rs to\n\
             {print:#x}",
            env!("UPSTREAM_DIR")
        );
        std::process::exit(1);
    }

    let cases = cases();
    let path = root.join("src/Cases.mw");
    std::fs::write(&path, &cases).unwrap();
    eprintln!("wrote {} ({} bytes)", path.display(), cases.len());
}

/// FNV-1a: stable across builds, which `DefaultHasher` does not promise.
fn fingerprint(text: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in text.bytes() {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

// --- encoding -----------------------------------------------------------------------

/// A number as `digits` base-64 digits, most significant first, each digit the
/// character `'0' + d`: `'0'` to `'o'`, one contiguous run of ASCII.
fn digits(out: &mut String, value: u64, digits: u32) {
    assert!(
        value < 1 << (6 * digits),
        "{value} does not fit in {digits} digits"
    );
    for k in (0..digits).rev() {
        out.push(char::from(b'0' + ((value >> (6 * k)) & 63) as u8));
    }
}

/// A string, as its length in bytes (3 digits) and then its bytes.
fn text(out: &mut String, s: &str) {
    digits(out, s.len() as u64, 3);
    out.push_str(s);
}

/// Bytes, as a string of their hex digits.
fn bytes(out: &mut String, b: &[u8]) {
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    text(out, &hex);
}

/// `text` as one Meadow string literal, broken with `\`-newline every `width`
/// characters. Only printable ASCII is written raw; a space that would start a
/// line is `\x20`, since a continuation drops leading whitespace.
fn long_literal(text: &str, width: usize) -> String {
    let mut out = String::with_capacity(text.len() + text.len() / width * 4 + 2);
    out.push('"');
    for (i, c) in text.chars().enumerate() {
        let line_start = i > 0 && i % width == 0;
        if line_start {
            out.push_str("\\\n    ");
        }
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '$' => out.push_str("\\$"),
            ' ' if line_start => out.push_str("\\x20"),
            ' '..='~' => out.push(c),
            _ => {
                let _ = write!(out, "\\u{{{:X}}}", u32::from(c));
            }
        }
    }
    out.push('"');
    out
}

// --- cases --------------------------------------------------------------------------

/// A small deterministic generator, so that the cases are the same on every run.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }

    fn bytes(&mut self, n: usize) -> Vec<u8> {
        (0..n).map(|_| self.next() as u8).collect()
    }
}

fn number(out: &mut String, n: u64) {
    text(out, &n.to_string());
}

fn cases() -> String {
    let mut rng = Rng(0xc3c3_2a4a_c0ff_ee42);
    let mut body = String::new();
    let mut counts = [0usize; 3];

    // Kind 0: bytes hashed whole, and in two pieces, from an initial state.
    for n in (0..300).chain([1000, 4096, 65536]) {
        let input = rng.bytes(n);
        let split = rng.below(n + 1);
        let init = if rng.below(3) == 0 {
            rng.next() as u32
        } else {
            0
        };
        counts[0] += 1;
        digits(&mut body, 0, 1);
        bytes(&mut body, &input);
        number(&mut body, u64::from(crc32fast::hash(&input)));
        digits(&mut body, split as u64, 3);
        let mut h = Hasher::new_with_initial(init);
        h.update(&input[..split]);
        h.update(&input[split..]);
        number(&mut body, u64::from(init));
        number(&mut body, u64::from(h.finalize()));
    }

    // Kind 1: two hashers combined.
    for _ in 0..400 {
        let a_len = rng.below(80);
        let a = rng.bytes(a_len);
        let b_len = rng.below(80);
        let b = rng.bytes(b_len);
        let mut ha = Hasher::new();
        ha.update(&a);
        let mut hb = Hasher::new();
        hb.update(&b);
        ha.combine(&hb);
        counts[1] += 1;
        digits(&mut body, 1, 1);
        bytes(&mut body, &a);
        bytes(&mut body, &b);
        number(&mut body, u64::from(ha.finalize()));
    }

    // Kind 2: the raw combination of two checksums over a length, including
    // lengths too long to hash.
    for _ in 0..400 {
        let c1 = rng.next() as u32;
        let c2 = rng.next() as u32;
        let len = match rng.below(4) {
            0 => rng.next() >> rng.below(64),
            1 => rng.below(100) as u64,
            2 => rng.next(),
            _ => 0,
        };
        let a = Hasher::new_with_initial_len(c1, 0);
        let b = Hasher::new_with_initial_len(c2, len);
        let mut combined = a.clone();
        combined.combine(&b);
        counts[2] += 1;
        digits(&mut body, 2, 1);
        number(&mut body, u64::from(c1));
        number(&mut body, u64::from(c2));
        number(&mut body, len);
        number(&mut body, u64::from(combined.finalize()));
    }

    let mut out = String::new();
    let _ = writeln!(
        out,
        "-- GENERATED by scripts/generate.sh from crc32fast {UPSTREAM_VERSION}.
-- Do not edit: run the script again instead.
--
-- Inputs, with the checksums the crate gives them, for `Tests.mw`: {} hashes,
-- {} combined hashers and {} raw combinations.
--
-- Copyright Sam Rijs and Alex Crichton, and the Meadow port's authors.
-- Dual-licensed under Apache-2.0 or MIT: see COPYRIGHT.

-- Each case starts with its kind (1 base-64 digit), and its fields follow in
-- the order `Tests.mw` reads them. A string is its length in bytes (3 digits)
-- and then its bytes; bytes are a string of their hex digits; numbers are
-- decimal strings.
@cfg(test)
@pub(pkg) def cases =
  {}",
        counts[0],
        counts[1],
        counts[2],
        long_literal(&body, 96)
    );
    out
}
