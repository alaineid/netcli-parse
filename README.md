# textfsm-rs

A TextFSM parsing engine written in Rust, designed for iOS integration via C FFI.

## Workspace Structure

| Crate | Purpose |
|---|---|
| `textfsm_core` | Pure-Rust parsing library (placeholder) |
| `textfsm_ffi` | C ABI layer for Swift / other language consumers |

## Build

```bash
cargo build --workspace
```

## Test

```bash
cargo test --workspace
```

## C Header

The FFI surface is declared in `include/textfsm.h`.
Link against the static or dynamic library produced by `textfsm_ffi`.

## iOS Integration

_Coming soon._
