# Contributing

Small fixes, bug reports and useful test scenes are welcome. For a larger change, open an issue first so we can talk through it.

## Reporting a problem

Use the [issue tracker](https://github.com/AlrikOlson/bevy_dlss5/issues). Include what you expected, what happened, and the smallest scene or steps that reproduce it. For rendering problems, include a screenshot and the relevant log output. GPU model, driver, OS, Bevy version and crate version matter here.

Also include the NR DLL version and which GPU architecture it was built for. Share its filename or hash, not the DLL itself.

## Making a change

Keep each PR focused on one change and explain why it helps. Add a regression test when a bug can be checked without a GPU. For shader or rendering changes, include before/after captures from the same scene, camera, resolution and exposure.

Run these from the crate directory:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo nextest run --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Use Windows and set `DLSS_SDK` to your NVIDIA DLSS SDK checkout before running the tests or building the example. The [README](README.md#requirements) covers the runtime DLL and executable setup.

`cargo nextest` is installed separately. If you cannot run a check or a GPU capture, say so in the PR and include what you did run.

Keep runtime DLLs and SDK files out of commits. The code is available under MIT or Apache-2.0; NVIDIA's binaries have their own terms.
