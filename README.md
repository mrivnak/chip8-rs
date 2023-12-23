# chip8.rs

![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/mrivnak/chip8-rs/check.yml)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/mrivnak/chip8-rs/test.yml?label=tests)
[![Coverage Status](https://coveralls.io/repos/github/mrivnak/chip8-rs/badge.svg?branch=main)](https://coveralls.io/github/mrivnak/chip8-rs?branch=main)
![GitHub release (latest SemVer)](https://img.shields.io/github/v/release/mrivnak/chip8-rs?display_name=tag&sort=semver)

![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![GitHub Actions](https://img.shields.io/badge/github%20actions-%232671E5.svg?style=for-the-badge&logo=githubactions&logoColor=white)
![Coveralls](https://img.shields.io/badge/coveralls-%23b94947.svg?style=for-the-badge&logo=coveralls&logoColor=white)
![Renovate](https://img.shields.io/badge/renovate-%230281a1?style=for-the-badge&logo=renovatebot&logoColor=white)

Chip-8 emulator written in Rust

## Getting Started

At the moment chip8.rs only supports running from the command line

```sh
# Linux/macOS
./chip8 <ROM>

# Windows
.\chip8.exe <ROM>
```

## Development

### Dependencies

- Rust

### Building

```sh
cargo build
```

### Running

```sh
cargo run -- <ROM>
```
