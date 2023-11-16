# Piped Slides

Creates slides for debugging advent of code.

Slides are are taken from the stdout of the provided subcommand.

They are delimited by the null byte.

```sh
$ cargo build --release
$ target/release/stdin-slides printf "ab\ncd\n\0ef\n\0gh\n\0"
```

![demonstration gif](./render1696735482867.gif)
