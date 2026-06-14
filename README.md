# xcode-simulator-manager-rs

A terminal UI for managing Xcode iOS simulators on macOS. List, sort, and bulk-delete simulators with live disk-usage scanning.

## Features

- Lists every simulator known to `xcrun simctl`, with name, runtime, state, UDID, and on-disk size.
- Background disk scan — sizes appear as each simulator's data directory finishes being walked.
- Multi-select and bulk delete; booted simulators are shut down first.
- Sort by name, runtime, state, or size.

## Requirements

- macOS with Xcode (or the Command Line Tools) installed — the tool shells out to `xcrun simctl`.

## Install

From crates.io:

```sh
cargo install xcode-simulator-manager-rs
```

Or download the prebuilt `aarch64-apple-darwin` tarball from the [latest release](https://github.com/uxsoft/xcode-simulator-manager-rs/releases/latest).

## Build from source

```sh
git clone https://github.com/uxsoft/xcode-simulator-manager-rs
cd xcode-simulator-manager-rs
cargo build --release
./target/release/xcode-simulator-manager-rs
```

## Usage

Run the binary with no arguments:

```sh
xcode-simulator-manager-rs
```

### Keys

| Key                | Action                                |
| ------------------ | ------------------------------------- |
| `j` / `↓`          | Move down                             |
| `k` / `↑`          | Move up                               |
| `PgDn` / `PgUp`    | Page down / up                        |
| `g` / `Home`       | Jump to top                           |
| `G` / `End`        | Jump to bottom                        |
| `Space`            | Toggle selection on the current row   |
| `s`                | Cycle sort column                     |
| `r`                | Refresh the simulator list            |
| `d`                | Delete selected simulators (confirm)  |
| `y` / `n`          | Confirm / cancel the delete prompt    |
| `q` / `Esc`        | Quit                                  |
| `Ctrl+C`           | Quit                                  |

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache 2.0](LICENSE-APACHE) at your option.
