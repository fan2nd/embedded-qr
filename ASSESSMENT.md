# embedded-qr Assessment

## Scope

This assessment uses two different evaluation lenses:

- Embedded-focused QR matrix generator
  - `no_std`
  - allocation-free
  - common payload support
  - final QR matrix output
- Full QR specification implementation
  - broader QR feature coverage
  - stronger spec-level completeness expectations

Unless otherwise noted, the primary evaluation target is the first lens.

## Current Snapshot

### High-level assessment

- Embedded-focused completeness: 7/10
- Embedded-focused robustness: 6/10
- Full-spec completeness: 4/10

### Why

- The main encoding pipeline is complete:
  - mode selection
  - ECC level selection
  - data encoding
  - Reed-Solomon ECC
  - interleaving
  - matrix placement
  - mask selection
- The implementation supports:
  - versions 1-40
  - ECC levels L/M/Q/H
  - Numeric, Alphanumeric, and Byte modes
- The main gaps are feature-surface and proof strength:
  - not all QR features are implemented
  - correctness evidence is still mostly example-driven rather than systematic

## Completeness Checklist

### Supported today

- Version support
  - `Version1` through `Version40`
  - Reference: [src/version.rs](/D:/MyGithub/embedded-qr/src/version.rs)
- ECC levels
  - `L`, `M`, `Q`, `H`
  - Reference: [src/types.rs](/D:/MyGithub/embedded-qr/src/types.rs)
- Encoding modes
  - `Numeric`
  - `Alphanumeric`
  - `Bytes`
  - Reference: [src/types.rs](/D:/MyGithub/embedded-qr/src/types.rs)
- Main pipeline
  - data encoding: [src/encoder.rs](/D:/MyGithub/embedded-qr/src/encoder.rs)
  - ECC generation: [src/ecc.rs](/D:/MyGithub/embedded-qr/src/ecc.rs)
  - interleaving: [src/interleave.rs](/D:/MyGithub/embedded-qr/src/interleave.rs)
  - matrix generation and mask choice: [src/matrix.rs](/D:/MyGithub/embedded-qr/src/matrix.rs)
  - public builder API: [src/builder.rs](/D:/MyGithub/embedded-qr/src/builder.rs)
- Automatic choices
  - mode selection
  - ECC level selection
  - best mask selection

### Missing or intentionally out of scope today

- Kanji mode
- ECI
- FNC1
- Structured Append
- Additional export/render helper formats beyond matrix access

### Evaluation questions

- Is the intended product scope “common embedded QR generation” or “full QR encoder”?
- Are the unsupported QR features documented clearly enough in README/docs?
- Is the current output surface enough for intended downstream users?

## Robustness Checklist

### Good signals already present

- ECC reference-value tests exist
  - Reference: [src/ecc.rs](/D:/MyGithub/embedded-qr/src/ecc.rs)
- Matrix snapshot tests exist
  - Reference: [src/matrix.rs](/D:/MyGithub/embedded-qr/src/matrix.rs)
- Interleave and version/alignment details have direct tests
  - References: [src/interleave.rs](/D:/MyGithub/embedded-qr/src/interleave.rs), [src/matrix.rs](/D:/MyGithub/embedded-qr/src/matrix.rs)

### Missing proof strength

- Broad version × ECC × mode boundary coverage
- Property-based testing
- Differential testing against a known-good QR implementation
- Fuzzing
- Dedicated `no_std` and embedded-target validation in CI
- Quantified stack and performance measurements

## Detailed Evaluation Matrix

### 1. Feature completeness

- Versions 1-40: pass
- ECC levels L/M/Q/H: pass
- Numeric/Alphanumeric/Byte modes: pass
- Full QR mode coverage: fail
- Main pipeline closure: pass
- Public API usability for the intended scope: pass

### 2. Boundary coverage

Review and add tests for each of these:

- Minimum payload
- Near-capacity payload
- Exactly full payload
- Overflow by 1 bit or byte
- Different ECC levels at the same version
- Small, medium, and large versions

Priority files:

- [src/builder.rs](/D:/MyGithub/embedded-qr/src/builder.rs)
- [src/encoder.rs](/D:/MyGithub/embedded-qr/src/encoder.rs)

### 3. Interleave correctness

Review and add tests for:

- Single-block layouts
- Multi-block layouts
- Mixed short/long block layouts
- High-version layouts near worst-case sizes

Priority file:

- [src/interleave.rs](/D:/MyGithub/embedded-qr/src/interleave.rs)

### 4. ECC correctness

Review and add tests for:

- Known reference ECC outputs across more levels and versions
- Different generator degrees
- Different block sizes

Priority file:

- [src/ecc.rs](/D:/MyGithub/embedded-qr/src/ecc.rs)

### 5. Matrix correctness

Review and add tests for:

- finder patterns
- timing patterns
- alignment patterns
- version bits
- format bits
- mask reproducibility
- final matrix snapshots across more versions and ECC levels

Priority file:

- [src/matrix.rs](/D:/MyGithub/embedded-qr/src/matrix.rs)

### 6. Error handling

Review and verify:

- invalid input path returns `QrError::DataInvalid`
- oversized payload returns `QrError::Overflow`
- no unexpected panics on normal invalid input paths

Priority files:

- [src/encoder.rs](/D:/MyGithub/embedded-qr/src/encoder.rs)
- [src/builder.rs](/D:/MyGithub/embedded-qr/src/builder.rs)

## Evidence Strength Checklist

### Current evidence

- Unit tests: moderate
- Snapshot tests: moderate
- Known-value tests: moderate

### Recommended additions

- Differential tests against a mature QR implementation
- Property tests for:
  - interleave permutation preserves the multiset of bytes
  - applying the same mask twice restores the original data region
  - successful encode never writes out of bounds
  - final matrix dimensions always match the selected version
- Fuzzing for:
  - `QrBuilder::build`
  - `EncodedData::encode`

## Embedded-specific Checklist

These matter more than average for this crate:

- `no_std` compilation checked in CI
- At least one embedded target build checked in CI
- Documented stack-sensitive paths
- Documented worst-case memory sizing
- Release-mode benchmarks for representative versions

## API and Maintainability Checklist

### Positive direction already visible

- Internal buffer details are being pulled behind `SealedVersion`
- Large buffer `Copy` exposure has been reduced
- Matrix mask search now uses fewer live buffers

### Remaining review questions

- Should additional internal types be hidden from the public API surface?
- Should benchmark or invariant tests be added before more algorithmic refactors?
- Should supported feature scope be documented explicitly in README?

## Recommended Next Steps

### Highest value

1. Add a documented feature matrix
2. Add systematic boundary tests for `mode × ecc × length`
3. Add differential tests against a known-good library
4. Add property tests for interleave and mask reversibility
5. Add `no_std` and embedded-target CI builds

### After that

1. Add stack and performance benchmarks
2. Expand matrix snapshots across more versions
3. Decide whether the crate aims to stay focused or grow toward fuller QR spec coverage

## Suggested Scoring Template

Use this table for future reassessment:

- Embedded-focused completeness: `0-10`
- Full-spec completeness: `0-10`
- Boundary-test coverage: `0-10`
- Correctness evidence strength: `0-10`
- Embedded readiness: `0-10`
- API cleanliness and maintainability: `0-10`

## Current Verdict

As an embedded-focused QR matrix generator, the crate is already a credible early production-style implementation, not just a toy. The next major improvement is not another core feature, but stronger correctness evidence and clearer scope documentation.
