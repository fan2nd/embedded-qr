# embedded-qr

`embedded-qr` is a `no_std`, allocation-free QR code generator for embedded Rust.

It uses typed QR versions such as `Version1` and `Version40`, automatically chooses the
most efficient encoding mode, and returns a traversable QR matrix that higher layers can
render however they like.

## Features

- `no_std`
- no heap allocation
- typed versions: `Version1` through `Version40`
- automatic mode detection: numeric, alphanumeric, or bytes
- optional manual mode override through `QrBuilder::with_mode`
- automatic error-correction selection by default
- optional manual ECC override through `QrBuilder::with_ecc_level`
- automatic mask evaluation
- public capacity queries through `embedded_qr::capacity`
- matrix output only, so rendering can stay application-specific

## Quick Start

```rust
use embedded_qr::{QrBuilder, Version1};

let qr = QrBuilder::<Version1>::new()
    .build(b"HELLO WORLD")
    .unwrap();

assert_eq!(qr.width(), 21);
assert!(qr.iter().any(|(_, _, dark)| dark));
```

If you want to pin the encoding mode and error-correction level instead of letting the
builder choose them automatically:

```rust
use embedded_qr::{EccLevel, EncodeMode, QrBuilder, Version4};

let qr = QrBuilder::<Version4>::new()
    .with_mode(EncodeMode::Bytes)
    .with_ecc_level(EccLevel::M)
    .build(b"hello from embedded")
    .unwrap();

assert_eq!(qr.ecc_level(), EccLevel::M);
```

## Output Model

This crate does not render PNG, SVG, or terminal graphics directly. Instead, it returns a
`QrMatrix<T>`:

- `width()` reports the side length in modules
- `get(x, y)` reads a single module
- `iter()` walks the matrix in row-major order

That makes it suitable for displays, printers, framebuffers, LEDs, and custom renderers.

## Capacity Queries

If you want to check whether a payload fits before building a matrix, use the public
capacity helpers:

```rust
use embedded_qr::{
    capacity,
    EccLevel,
    EncodeMode,
    Version1,
};

assert!(capacity::fits::<Version1>(EncodeMode::Bytes, 17, EccLevel::L));
assert!(!capacity::fits::<Version1>(EncodeMode::Bytes, 18, EccLevel::L));

assert_eq!(
    capacity::max_payload_len::<Version1>(EncodeMode::Alphanumeric, EccLevel::M),
    20
);

let info = capacity::info::<Version1>(EccLevel::Q);
assert_eq!(info.data_codewords, 13);
assert_eq!(info.ecc_codewords, 13);
```

## Example

The repository includes a `std` example that prints versioned QR codes in the terminal:

```bash
cargo run --example std
```

## Current Scope

- supports QR versions `1` through `40`
- supports numeric, alphanumeric, and byte mode encoding
- chooses the best mask automatically
- chooses the strongest fitting ECC level automatically unless overridden
- exposes capacity queries for version/mode/ECC planning

The crate currently focuses on producing QR matrices. Rendering and image export are left to
the application layer.

## Improvement Opportunities

The current implementation is already small, `no_std`, and allocation-free, but a few
follow-up areas would make it more complete and easier to adopt:

- add more encoding features such as Kanji mode, ECI, and GS1/FNC1 support
- add mixed-mode segmentation so payloads can use more compact combinations than a single
  global mode choice
- expand compatibility validation with more cross-version reference vectors, property tests,
  and fuzzing around mode selection, interleaving, and matrix generation
- add more integration examples for common embedded display stacks or renderer crates so the
  matrix output is easier to consume in real applications

If you are evaluating the crate today, the biggest practical limitation is encoding scope:
numeric, alphanumeric, and byte mode are supported, but more advanced QR features are not yet
implemented.
