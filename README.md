# Iron Bin

**Iron Bin**, a CLI for the [Freedesktop trash](https://specifications.freedesktop.org/trash-spec/) written in Rust :crab:.

## Usage

Iron Bin consists of a single command: `trash`.

To list the files in the trash:

```shell
trash list
```

or

```shell
trash ls
```

The output is similar to that of the `ls` command, but the original paths are always absolute:

```
/home/johndoe/Documents/Invoice.pdf
/home/johndoe/Documents/Notes.doc
/home/johndoe/Downloads/funny.jpg
```

To display more details, add the `-v` option:

```shell
trash list -v
```

The output is similar to that of the `ls -l` command, but the columns are a bit different:

```
total 3
       size  deletion time             original path
   36196652  Mon Sep  8 22:11:09 2025  /home/johndoe/Documents/Invoice.pdf
    4668381  Mon Sep  8 22:11:09 2025  /home/johndoe/Documents/Notes.doc
     568731  Tue Sep  9 09:24:37 2025  /home/johndoe/Downloads/funny.jpg
```

**Note:** The same file may appear several times in the trash if it was put there at several different times.

To put a file in the trash:

```shell
trash put Downloads/message.txt
```

To prompt for confirmation before each file, add the `-i` option (similar to the `rm -i` command):

```shell
trash put -i Downloads/*
```

To display a summary, add the `-v` option:

```shell
trash put Downloads/message.txt
```

```
trashed /home/johndoe/Downloads/message.txt on Mon Nov 24 19:02:16 2025
total 1 trashed
```

To restore a file from the trash:

```shell
trash restore /home/johndoe/Documents/Notes.doc
```

You can restore several files at once:

```shell
trash restore /home/johndoe/Documents/Notes.doc /home/johndoe/Downloads/funny.jpg
```

**Note:** If the same file was put in the trash several times, the most recent version is restored.

To prompt for confirmation before each file, add the `-i` option (similar to the `mv -i` command):

```shell
trash restore -i /home/johndoe/Documents/Notes.doc
```

To display a summary, add the `-v` option:

```shell
trash restore -v /home/johndoe/Documents/Notes.doc
```

```
restored /home/johndoe/Documents/Notes.doc trashed on Mon Sep  8 22:11:09 2025
total 1 restored
```

To empty the trash:

```shell
trash empty
```

A confirmation prompt is displayed.
To skip it, add the `-f` option:

```shell
trash empty -f
```

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
