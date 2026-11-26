# Contributing to OpenSir

Thanks for helping. OpenSir writes to the flash memory of real controllers, so changes that touch the protocol or the write path get extra scrutiny.

## Build and check

Windows with the MSVC Rust toolchain, Node 22 and the WebView2 runtime (see the README).

```sh
npm ci
npm run check                                        # svelte-check, must be 0 errors / 0 warnings
cargo test --workspace                               # codecs, session, golden vectors
cargo clippy --workspace --all-targets -- -D warnings
npm run tauri dev                                    # run the app
npm run dev                                          # UI only, simulated controllers, in a browser
```

## Commits

Use [Conventional Commits](https://www.conventionalcommits.org/) (`feat(ui): …`, `fix(core): …`, `docs(protocol): …`). Keep one logical change per commit.

## Protocol facts only

- Never commit vendor code, minified or decompiled output, vendor binaries, logs with personal data, or file/line references into the vendor app.
- Keep reverse-engineering scratch in `re/` (gitignored).
- Docs in `docs/protocol` state facts: bytes, offsets, ranges, observed behaviour, captures from your own hardware.

## Testing writes on hardware

- Close GameSir Connect first.
- Back up every block before writing.
- Test on a profile that is not the active one, and restore it byte for byte afterwards.
- A model stays read-only (`Session.writable`) until its write path has passed that test on real hardware. Say in the pull request which controller, connection and firmware you used.

## Reporting

Bugs: open an issue with the template. Anything that could write bad data to a controller: see [SECURITY.md](SECURITY.md).
