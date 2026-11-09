# Temple3

Temple3 is a Rust library and CLI for the laboratory protocol for witnessed
state transitions. It provides canonical commitments, predicate and covenant
evaluation, signed witnesses, authenticated journal replay, coordination and
federation verification.

## Build and verify

Use Rust and Cargo with Rust 2024 support.

```sh
cargo build --locked --release
cargo test --locked --release
cargo run --locked --release --example transfer
```

The transfer example signs and verifies a laboratory transition with a public
demonstration seed. The native CLI reads JSON on stdin for commands such as
`canonical`, `genesis`, `transition` and `replay`. Run `temple3 --help`
for the command list.

### External conformance

The four shared vector files and workspace captures are maintained separately.
To run the 250 shared vectors in Gates 1–4 and the fixture-backed Rust tests,
point these commands at the external corpus root:

```sh
cargo run --locked --release -- verify "$CORPUS_ROOT"
TEMPLE3_CORPUS_ROOT="$CORPUS_ROOT" cargo test --locked --release -- --ignored
```

Add `--legacy` to the verifier command when the corpus also contains the
Gate 5 archaeology files. Authority recognizers compare canonical
fingerprints of their expected witness fields. The underlying workspace
receipts stay with the external integration.

## Use as a library

Add a Git dependency pinned to the commit you intend to use.

```toml
temple3 = { git = "https://github.com/aaron-alderman/temple3.git", rev = "<commit-sha>" }
```

- `kernel` creates genesis state and evaluates proposed transitions.
- `identity` signs envelopes and verifies or replays authenticated history.
- `journal::FileJournal` handles the cooperating-writer filesystem journal.
- `replica::Replica` retains durable vote locks.
- `coordination` and `federation` verify quorum and foreign-chain evidence.

A caller supplies the signing seed and the independently trusted Temple ID.
The Workgarden journal adapter uses this library at its application boundary.
Encounter core is a separate host with its own application declaration and
image contracts.
