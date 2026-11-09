use serde_json::{Value, json};
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc, Barrier,
        atomic::{AtomicU64, Ordering},
    },
};
use temple3::{commitment, identity, journal::FileJournal, kernel, pilot};
static IDS: AtomicU64 = AtomicU64::new(0);
fn directory() -> PathBuf {
    let p = std::env::temp_dir()
        .join(format!(
            "rust-test-{}-{}",
            std::process::id(),
            IDS.fetch_add(1, Ordering::Relaxed)
        ));
    fs::create_dir_all(&p).unwrap();
    p
}
fn proposal(r: &temple3::replica::Replica, id: &str) -> Value {
    let s = r.state(None).unwrap();
    let v = if pilot::current_cell(&s).unwrap()["value"]["value"] == "green" {
        "amber"
    } else {
        "green"
    };
    pilot::propose(
        &s,
        &json!({"value":v,"basisStateRoot":s["temple"]["stateRoot"],"requestId":id}),
    )
    .unwrap()
}
#[test]
fn repeated_replacements_preserve_unrelated_cells_and_prevent_aba() {
    let p = directory();
    let r = pilot::initialize(&p).unwrap();
    let before = r.state(None).unwrap();
    let old = proposal(&r, "first");
    for i in 0..8 {
        let proposal = proposal(&r, &format!("edit-{i}"));
        assert_eq!(pilot::commit(&r, &proposal).unwrap()["status"], "finalized");
    }
    let after = pilot::open(&p, None).unwrap().state(None).unwrap();
    assert_eq!(
        pilot::current_cell(&after).unwrap()["value"]["value"],
        "green"
    );
    assert_ne!(before["temple"]["stateRoot"], after["temple"]["stateRoot"]);
    assert_eq!(
        after["temple"]["state"]["cells"].as_array().unwrap().len(),
        2
    );
    let note = |s: &Value| {
        s["temple"]["state"]["cells"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["value"]["kind"] == "pilot-note")
            .unwrap()
            .clone()
    };
    assert_eq!(note(&before), note(&after));
    assert_eq!(pilot::commit(&r, &old).unwrap()["reason"], "stale-basis");
}
#[test]
fn durable_vote_retries_and_competing_process_handles_cannot_double_vote() {
    let p = directory();
    let r = pilot::initialize(&p).unwrap();
    let a = proposal(&r, "a");
    let b = proposal(&r, "b");
    let vote = r.cast_vote(&a).unwrap();
    let reopened = pilot::open(&p, None).unwrap();
    assert_eq!(reopened.cast_vote(&a).unwrap(), vote);
    assert_eq!(reopened.cast_vote(&b).unwrap_err().code, "vote-locked");
    assert_eq!(reopened.resolve().unwrap()["status"], "finalized");
    assert_eq!(pilot::commit(&reopened, &a).unwrap()["status"], "replay");
}
#[test]
fn racing_writers_release_at_most_one_height_vote() {
    let p = directory();
    let r = pilot::initialize(&p).unwrap();
    let a = proposal(&r, "race-a");
    let b = proposal(&r, "race-b");
    let barrier = Arc::new(Barrier::new(2));
    let threads: Vec<_> = [a, b]
        .into_iter()
        .map(|proposal| {
            let dir = p.clone();
            let b = barrier.clone();
            std::thread::spawn(move || {
                let replica = pilot::open(&dir, None).unwrap();
                b.wait();
                replica.cast_vote(&proposal)
            })
        })
        .collect();
    let results: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
    assert_eq!(results.iter().filter(|r| r.is_ok()).count(), 1);
    assert_eq!(FileJournal::new(&p).read().unwrap().len(), 2);
}
#[test]
fn uncertain_publication_recovers_without_overwriting_or_releasing_second_vote() {
    for phase in [
        "stage-written",
        "stage-synced",
        "published",
        "stage-removed",
    ] {
        let p = directory();
        let r = pilot::initialize(&p).unwrap();
        let a = proposal(&r, "original");
        let b = proposal(&r, "other");
        let before = fs::read(p.join("000000000000.json")).unwrap();
        assert!(
            r.cast_vote_observed(&a, |at| if at == phase {
                Err(temple3::Error {
                    code: "injected-io".into(),
                })
            } else {
                Ok(())
            })
            .is_err()
        );
        assert_eq!(fs::read(p.join("000000000000.json")).unwrap(), before);
        let reopened = pilot::open(&p, None).unwrap();
        let result = reopened.cast_vote(&b);
        if ["published", "stage-removed"].contains(&phase) {
            assert_eq!(result.unwrap_err().code, "vote-locked");
        } else {
            assert!(result.is_ok());
        }
    }
}
#[test]
fn stale_lock_corruption_and_external_tip_fail_closed() {
    let p = directory();
    let r = pilot::initialize(&p).unwrap();
    let proposal = proposal(&r, "x");
    r.cast_vote(&proposal).unwrap();
    let tip = r.state(None).unwrap()["tip"].clone();
    fs::write(p.join(".writer-lock"), b"possibly alive").unwrap();
    assert_eq!(r.cast_vote(&proposal).unwrap_err().code, "journal-busy");
    fs::remove_file(p.join(".writer-lock")).unwrap();
    let path = p.join("000000000001.json");
    let bytes = fs::read(&path).unwrap();
    fs::write(&path, b"broken").unwrap();
    assert!(r.state(None).is_err());
    fs::write(&path, &bytes).unwrap();
    fs::remove_file(&path).unwrap();
    assert_eq!(
        r.state(Some(&tip)).unwrap_err().code,
        "journal-tip-mismatch"
    );
}
#[test]
fn replacement_covenant_denies_other_effects_even_with_editor_signature() {
    let p = directory();
    let r = pilot::initialize(&p).unwrap();
    let s = r.state(None).unwrap();
    let valid = proposal(&r, "x");
    let extra = commitment::cell_for(&json!("injected")).unwrap();
    for effects in [
        json!({"nextPredicate":{"op":"literal","value":true}}),
        json!({"consume":s["temple"]["state"]["cells"].as_array().unwrap().iter().map(|c|&c["id"]).collect::<Vec<_>>(),"create":[extra]}),
    ] {
        let d = kernel::build_delta(&s["temple"], &effects).unwrap();
        let signed=identity::sign_envelope("proposal",&json!({"templeId":s["temple"]["templeId"],"actor":"editor","requestId":"attack","delta":d,"context":{"application":pilot::CONTEXT}}),&pilot::SEED).unwrap();
        assert_eq!(pilot::preview(&s, &signed).unwrap()["status"], "rejected");
    }
    assert_eq!(pilot::preview(&s, &valid).unwrap()["status"], "ready");
}

#[test]
fn cached_state_matches_fresh_replay_after_external_writes_and_branch_replacement() {
    let p = directory();
    let r = pilot::initialize(&p).unwrap();
    let other = pilot::open(&p, None).unwrap();
    let a = proposal(&r, "external-a");
    other.cast_vote(&a).unwrap();
    let pending = r.state(None).unwrap();
    assert_eq!(pending["votes"].as_array().unwrap().len(), 1);
    other.resolve().unwrap();
    let finalized = r.state(None).unwrap();
    assert_eq!(
        finalized,
        pilot::open(&p, None).unwrap().state(None).unwrap()
    );

    // Simulate an owner replacing a valid suffix. Without an external tip this
    // is another valid history, so cache reuse must not retain the old branch.
    let alternate = directory();
    let branch = pilot::initialize(&alternate).unwrap();
    let b = proposal(&branch, "external-b");
    pilot::commit(&branch, &b).unwrap();
    for index in 1..=2 {
        let name = format!("{index:012}.json");
        fs::copy(alternate.join(&name), p.join(&name)).unwrap();
    }
    assert_eq!(
        r.state(Some(&finalized["tip"])).unwrap_err().code,
        "journal-tip-mismatch"
    );
    let replaced = r.state(None).unwrap();
    assert_ne!(replaced["finalityId"], finalized["finalityId"]);
    assert_eq!(
        replaced,
        pilot::open(&p, None).unwrap().state(None).unwrap()
    );
}

#[test]
fn late_competing_quorum_halts_even_when_first_quorum_is_cached() {
    let p = directory();
    let r = pilot::initialize(&p).unwrap();
    let a = proposal(&r, "first-quorum");
    pilot::commit(&r, &a).unwrap();
    assert_eq!(r.resolve().unwrap()["status"], "unresolved");
    // A second fixture replica deliberately uses the same public fixture key
    // on a separate journal to produce cryptographic equivocation evidence.
    let alternate = directory();
    let faulty = pilot::initialize(&alternate).unwrap();
    let b = proposal(&faulty, "competing-quorum");
    let vote = faulty.cast_vote(&b).unwrap();
    r.receive_vote(&b, &vote).unwrap();
    assert_eq!(r.resolve().unwrap()["status"], "halted");
    let halted = r.state(None).unwrap();
    assert_eq!(halted["halted"], true);
    assert_eq!(halted["conflicts"].as_array().unwrap().len(), 1);
    assert_eq!(halted, pilot::open(&p, None).unwrap().state(None).unwrap());
    assert_eq!(r.resolve().unwrap_err().code, "halted-coordination");
}

#[test]
fn private_pilot_genesis_cannot_fall_back_to_public_fixture_authority() {
    let p = directory();
    let seed = [91; 32]; // Different public test key, never an operational credential.
    let r = pilot::initialize_with_seed(&p, &seed).unwrap();
    assert!(pilot::open(&p, None).is_err());
    let state = r.state(None).unwrap();
    let input = json!({"value":"amber","basisStateRoot":state["temple"]["stateRoot"],"requestId":"private-mode"});
    let wrong = pilot::propose(&state, &input).unwrap();
    assert!(pilot::preview(&state, &wrong).is_err());
    let valid = pilot::propose_with_seed(&state, &input, &seed).unwrap();
    assert_eq!(pilot::commit(&r, &valid).unwrap()["status"], "finalized");
    assert_eq!(
        pilot::open_with_seed(&p, None, &seed)
            .unwrap()
            .state(None)
            .unwrap(),
        r.state(None).unwrap()
    );
}
