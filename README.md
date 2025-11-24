# Iron Bin

**Iron Bin**, a CLI for the [Freedesktop trash](https://specifications.freedesktop.org/trash-spec/) written in Rust :crab:.

## Limitations

- Only the home trash is supported
- Only UTF-8 file names are supported

## Changelog

See [CHANGELOG](CHANGELOG.md).

## License

Iron Bin is licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).
See [LICENSE](LICENSE).

## Similar projects

- [trash-cli](https://github.com/andreafrancia/trash-cli) (Python)

## Development

### Overview

Iron Bin is written in Rust.
This project is a [Cargo](https://doc.rust-lang.org/cargo/) workspace where the following crates are the most important:

- [iron-bin](crates/iron-bin): the library that provides the `Trash` struct
- [trash](crates/trash): the `trash` binary, which is essentially a CLI for the `iron-bin` library

### Git hooks

It is recommended to use the Git hooks provided with this project.
This can be easily achieved with [Smee](https://github.com/lpireyn/smee).
