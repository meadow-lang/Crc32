# crc32

The CRC-32 checksum used by zlib, gzip, PNG and Ethernet (IEEE 802.3), for
[Meadow](https://github.com/mcdearman/meadow).

This package is a port of Rust's
[`crc32fast`](https://github.com/srijs/rust-crc32fast) 1.5.2.

## Install

```sh
meadow add mcdearman/MeadowCrc32
```

## Use

```meadow
use Crc32 (hashString, newHasher, update, finalize)

def main =
  ( hashString "hello world",              -- 222957957 (0x0D4A1185)
    finalize (update (update newHasher (stringToBytes "hello")) (stringToBytes " world"))
  )
```

| function | |
|---|---|
| `hash bytes`, `hashString s` | the checksum, as a `UInt32` |
| `newHasher`, `newWithInitial crc`, `newWithInitialLen crc len` | a `Hasher` for computing a checksum a piece at a time |
| `update hasher bytes`, `finalize hasher`, `reset hasher` | add bytes, read the checksum, start again |
| `combine a b` | the checksum of `a`'s bytes followed by `b`'s, computed from the two checksums alone |
| `combineChecksums crc1 crc2 len2` | the same, given the second checksum's length |

`Hasher` is an immutable record, so `update` returns a new one.

## How it's made

`src/Crc32.mw` translates the crate's portable implementation, including its
checksum-combining arithmetic. It uses the byte-at-a-time table rather than
the crate's slicing-by-16, which gives the same results. **`src/Cases.mw`** is
generated test data:

- 303 inputs, each hashed whole and in two pieces from a random starting
  checksum;
- 400 combined hashers;
- 400 raw combinations over lengths up to 2⁶⁴.

Every expected checksum comes from calling the crate. Run
`scripts/generate.sh` to regenerate; it needs a Rust toolchain.

## Licence

Dual-licensed under [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your
option, like the crate. See [COPYRIGHT](COPYRIGHT).
