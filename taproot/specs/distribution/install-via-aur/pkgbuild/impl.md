# Implementation: PKGBUILD

## Behaviour
../usecase.md

## Design Decisions
- Package named `rpnpad-bin` following AUR convention for pre-built binary packages — avoids conflict with a potential future `rpnpad` source package; users install with `yay -S rpnpad-bin`
- PKGBUILD pulls the pre-built `x86_64-unknown-linux-gnu` binary archive from the GitHub Release — no Rust toolchain required on the user's machine; fast install
- `arch=('x86_64')` only — cargo-dist currently builds only x86_64 for Linux; if `aarch64-unknown-linux-gnu` is added to `Cargo.toml` targets, a second `source_aarch64` stanza can be added
- `sha256sums` must be updated on every release; an `update-aur.sh` helper script automates fetching the new checksum and bumping `pkgver`/`pkgrel`
- The PKGBUILD lives in `channels/aur/` — the maintainer pushes to the AUR remote (`ssh://aur@aur.archlinux.org/rpnpad-bin.git`) from that directory
- LICENSE file installed alongside the binary per AUR packaging guidelines

## Source Files
- `channels/aur/PKGBUILD` — AUR package definition: version, source URL, checksum, install steps
- `channels/aur/.SRCINFO` — machine-readable package metadata (generated from PKGBUILD via `makepkg --printsrcinfo`); required by AUR
- `channels/aur/update-aur.sh` — helper script to update pkgver, sha256sum, and regenerate .SRCINFO for a new release

## Commits
- placeholder
- `b19ec0fec2f3dccf02faa8f6c1eceaabd4f3bf96` — (auto-linked by taproot link-commits)

## Tests
This is a packaging configuration. Integration tests require an Arch Linux environment:

- **AC-1** (fresh install): `yay -S rpnpad-bin` in a clean Arch VM; verify `rpnpad` on PATH
- **AC-2** (upgrade): install old version, update PKGBUILD to new version, run `yay -S rpnpad-bin`; verify new version active
- **AC-3** (manual makepkg): `cd channels/aur && makepkg -si`; verify install identical to AUR helper flow
- **AC-4** (checksum mismatch): corrupt sha256sum in PKGBUILD; verify makepkg aborts with integrity error
- **AC-5** (wrong arch): add `aarch64` to arch array without a source for it; verify failure

## DoR Resolutions

## Status
- **State:** complete
- **Created:** 2026-03-25
- **Last verified:** 2026-03-25

## Notes
- First submission to AUR requires creating an account at aur.archlinux.org and adding an SSH key. The AUR remote for a new package is `ssh://aur@aur.archlinux.org/rpnpad-bin.git` (the repo is created automatically on first push).
- After each GitHub Release, run `channels/aur/update-aur.sh <version>` then `git push` to the AUR remote.

## DoD Resolutions
- condition: document-current | note: README has no aur/ path references — channels/aur/ rename is maintainer-facing only, not user-visible; no README update required | resolved: 2026-03-28T14:48:02.973Z
