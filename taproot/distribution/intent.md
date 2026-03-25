# Intent: Zero-Friction Installation for Linux and macOS

## Stakeholders
- CLI power user: any developer or power user who discovers rpnpad — their interest is installing and running it in under a minute, without setting up a Rust toolchain
- Maintainer: the solo developer — their interest is automated releases that reach users without manual packaging work on every version bump

## Goal
Enable users on Linux and macOS to install rpnpad with a single command — no Rust toolchain required. When a new version is tagged, the release pipeline runs automatically and produces installable artifacts, so users always have a frictionless path to the latest build.

## Success Criteria
- [ ] A macOS user can install rpnpad with `brew install` and have it on PATH
- [ ] A Linux user can install rpnpad with a `curl | sh` script and have it on PATH
- [ ] Pushing a git tag triggers the full release pipeline without manual steps
- [ ] `cargo install rpnpad` continues to work for users who do have Rust
- [ ] Release artifacts are reproducibly built from source in CI

## Constraints
- Must be maintainable by a solo developer with no dedicated DevOps support
- Binaries must be produced in CI from source (no pre-built binaries committed to the repo)
- Windows is out of scope for this intent

## Behaviours <!-- taproot-managed -->
- [Maintainer publishes a release via cargo-dist](./cargo-dist-release-pipeline/usecase.md)
- [User installs rpnpad via Homebrew](./install-via-homebrew/usecase.md)
- [User installs rpnpad via curl installer](./install-via-curl/usecase.md)
- [User reads the project README to discover and learn rpnpad](./project-readme/usecase.md)
- [User installs rpnpad via AUR](./install-via-aur/usecase.md)
- [User installs rpnpad via Snap Store](./install-via-snap/usecase.md)

## Status
- **State:** draft
- **Created:** 2026-03-24
- **Last reviewed:** 2026-03-24

## Notes
- cargo-dist is the primary vehicle: it generates GitHub Actions workflows, a Homebrew tap formula, and a shell installer from a single `cargo dist init` invocation
- Out of scope: apt/deb/rpm packaging, Nix, Windows
