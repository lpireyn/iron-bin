# Iron Bin

**Iron Bin**, a CLI for the [Freedesktop trash](https://specifications.freedesktop.org/trash-spec/) written in Rust :crab:.

## Limitations

- Invalid UTF-8 characters in file and directory names are not supported
- Only the home trash is supported

## Changelog

See [CHANGELOG](CHANGELOG.md).

## License

Iron Bin is licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).
See [LICENSE](LICENSE).

## Development

### Git hooks

It is recommended to use the Git hooks provided in the `.githooks` directory.
This can be achieved with:

``` shell
git config set core.hooksPath .githooks
```
