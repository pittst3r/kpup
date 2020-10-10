# `kpup`

<u>K</u>ill <u>p</u>rocess <u>u</u>sing <u>p</u>ort, for *nix.

Tired of using `lsof` to find the PID of an orphan process, then `kill`ing
with the PID? This tool simply kills the process that is using the given port.
Sends a `SIGINT` by default, while the `-f`/`--force` flag sends a `SIGKILL`.

## Installation

```
$ brew install pittst3r/formulae/kpup
```

## Usage

```
USAGE:
    kpup [FLAGS] <port>

FLAGS:
    -f, --force      Send SIGKILL instead of SIGINT
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <port>    port
```
