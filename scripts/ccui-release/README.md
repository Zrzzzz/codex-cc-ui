# codex-claude-code-ui

A compact native Codex terminal interface, inspired by Claude Code.

Project: https://github.com/Zrzzzz/codex-cc-ui

## Requirements

This preview package supports Linux x86_64 with glibc 2.39 or newer,
OpenSSL 3 and libgcc. It does not support Alpine/musl, macOS or Windows.

## Run

After extracting the standalone archive, keep its directory structure intact
and run `./codex-cc`. When installed through npm, run `codex-cc` from your PATH.
The package does not replace an existing `codex` command. Existing Codex
configuration and credentials are reused.

## Preview Status

The first preview uses a stripped, unoptimized development build. The native
CLI still reports the upstream development version `0.0.0`; the CCUI package
version and source commit are recorded in `BUILD-INFO.json`.

The Code Mode host, bwrap, patched zsh and ripgrep are redistributed from the
official Codex 0.153.4 Linux x86_64 package. The Code Mode host is included
because the V8 archive needed to build it from this checkout is unavailable.
Basic compatibility checks do not establish full cross-version compatibility.

Checksums and companion versions are recorded in `BUILD-INFO.json`.
Apache-2.0; see `LICENSE` and `NOTICE`.
