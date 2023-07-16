# Contributing

This project is implemented in [Rust](https://www.rust-lang.org/).

Following sections are some tips to ease development cycle, especially if you code on MacOS.

## Compile and lint locally

Install toolchains:
```bash
brew tap messense/macos-cross-toolchains
brew install arm-unknown-linux-gnueabihf # (ARMv6 for RPi Zero)
brew install aarch64-unknown-linux-gnu # (ARMv8 for RPi Zero 2 and RPi 4)
```

Install Rust targets:
```bash
rustup target add arm-unknown-linux-gnueabihf
rustup target add aarch64-unknown-linux-gnu
```

Add following lines in `~/.cargo/config`:
```
[target.arm-unknown-linux-gnueabihf]
linker = "arm-unknown-linux-gnueabihf-gcc"

[target.aarch64-unknown-linux-gnu]
linker = "aarch64-unknown-linux-gnu-gcc"
```

You can now run eg. `cargo build --target aarch64-unknown-linux-gnu`.

## Run binary or tests remotely

Add following lines in `~/.cargo/config`:
```
[target.'cfg(any(target_arch = "arm", target_arch = "aarch64"))']
runner = "pi-runner.sh"
```

Create file `~/.cargo/bin/pi-runner.sh` with the following content. Don't forget to make this file executable!
```bash
#!/bin/bash

[[ -z "${TARGET_HOST}" ]] && echo "Please set TARGET_HOST envvar" && exit 1

BINARY_PATH="$1"
BINARY_FILE=$(basename "${BINARY_PATH}")
PATH_PREFIX="/tmp"

scp "${BINARY_PATH}" pi@"$TARGET_HOST":"${PATH_PREFIX}" &> /dev/null
ssh -o LogLevel=QUIET -t -t pi@"$TARGET_HOST" "${PATH_PREFIX}/${BINARY_FILE} ${@:2}"
```

You can now run eg. `TARGET_HOST=192.168.xxx.xxx cargo test --target aarch64-unknown-linux-gnu`, with `TARGET_HOST` an IP pointing to an ssh-enabled RPi 4.

If you use [Oh My Zsh](https://ohmyz.sh/), you can [enable dotenv plugin](https://github.com/ohmyzsh/ohmyzsh/tree/master/plugins/dotenv).
This allows you to autoset envvars while working in the project directory, saving extra `cargo` parameters:

```bash
$ cat .env
export CARGO_BUILD_TARGET=aarch64-unknown-linux-gnu
export TARGET_HOST=192.168.xxx.xxx
```

You can now run just eg. `cargo test` or `cargo run --example pages-screenshot` seamlessly.
