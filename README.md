# VMP

VMP builds and installs selected Python releases under `~/.vmp`.

```sh
vmp install 3.14
vmp use 3.14
vmp list installed
```

Python source archives are downloaded over HTTPS together with the release's
Sigstore bundle. VMP verifies both new and cached archives against Sigstore's
production trust root and the CPython release-manager identity pinned for that
minor series before extracting or executing build scripts. Unsupported release
series and failed identity, transparency-log, certificate, redirect, or bundle
validation fail closed.

Native verification:

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build
cargo test
```
