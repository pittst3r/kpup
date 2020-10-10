# `kpup`

**K**ill **p**rocess **u**sing **p**ort, for *nix.

Hate it when a process using a port gets orphaned? Tired of using `lsof` to find
the PID, then `kill`ing with the PID? This tool kills the process that is
listening on the given port in one go. Sends a `SIGINT` by default, while the
`-f` flag sends a `SIGKILL`.

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
    <port>    the port on which the process is listening
```
