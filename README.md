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

### Application proof relations

An adapter can pass a trusted `federation::RelationVerifier` to
`kernel::evaluate_transition_with_relations` and
`identity::replay_with_relations`. The verifier receives the relation name,
public inputs and evidence after Temple3 checks the statement and envelope.
It returns `None` for unrecognized relations. Use the same deterministic,
bounded implementation for admission and replay. Temple3 retains its built-in
verification rules and proof budget.

The default resolver covers protocol proofs for foreign finality, federation
readiness, private openings and state disclosure. Application proof relations
require an explicit adapter. The same resolver can be supplied for compact
transitions, coordination candidates, certificates, and journal replay.
`Replica::create_with_relations` and `Replica::open_with_relations` keep
the resolver fixed for the lifetime of a replica, including cached replay.

### Shadow replay

The Sourcery shadow CLI lives in the Temple3 research workspace. Build its
`implementations/rust` crate with `cargo build --locked --release --bin temple3`.
Use the absolute path to that trusted research executable:

```sh
export TEMPLE3_SHADOW_CLI=/absolute/path/to/research/temple3/implementations/rust/target/release/temple3
temple3 runtime-events < events.json
temple3 runtime-journal < journal.json
```

These aliases forward stdin, stdout, stderr and the exit status. The
research CLI can also run the same commands directly.
