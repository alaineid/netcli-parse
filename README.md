# netcli-parse

Network CLI output parsing library — knows platforms, commands, and templates so
callers don't have to.

**This library does NOT connect to devices.** It takes raw CLI output as a
string and returns structured records (JSON).

## Workspace

| Crate | Purpose |
|---|---|
| `netcli_core` | Parsing SDK: platform taxonomy, command keys, template registry, normalization |
| `netcli_ffi` | Thin C ABI wrapper (`netcli_parse_json` / `netcli_free`) for Swift and other languages |

## Quick start

```bash
cargo build --workspace
cargo test  --workspace
```

## C / Swift integration

Link against the static or dynamic library produced by `netcli_ffi` and include
`include/netcli_parse.h`.

```c
#include "netcli_parse.h"

const char *json = netcli_parse_json("cisco_ios", "show_version", raw_output);
// use json …
netcli_free(json);
```

## Supported platforms

| Slug | Aliases |
|---|---|
| `cisco_ios` | `ios` |
| `cisco_nxos` | `nxos`, `nx_os` |
| `cisco_iosxr` | `iosxr`, `ios_xr` |
| `juniper_junos` | `junos` |
| `arista_eos` | `eos` |
| `drivenets_dnos` | `dnos`, `drivenets` |

## Command keys

`show_version`, `show_interfaces_brief`, `show_inventory`, `show_bgp_summary`,
`show_ip_route`, `show_lldp_neighbors`

## Roadmap

- **Phase 1** (current): Input validation, JSON envelope, platform/command taxonomy — parsing returns empty records (stub).
- **Phase 2**: Integrate a TextFSM engine crate, load templates from `resources/templates/`, return real parsed records.
- **Phase 3**: Optional field normalization into a canonical schema per command key.
