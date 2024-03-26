
set positional-arguments

build:
    cargo build --release

run *args:
    #!/usr/bin/env bash
    cargo run -- "$@"
