use serde_json::{Value, json};
use std::{
    env,
    io::{self, Read},
    path::PathBuf,
    process::ExitCode,
};
use temple3::{canonical, coordination, federation, identity, kernel, vectors};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().skip(1).collect();
    let command = args.first().map(String::as_str).unwrap_or("help");
    if command == "help" || command == "--help" {
        println!(
            "Temple3 native Rust laboratory\n\n  temple3 verify CORPUS_ROOT\n  temple3 <command> < input.json\n\nCommands: canonical, decode, identify, genesis, build-delta, transition,\nproof-transition, verify-signature, replay, coordination-replay,\nverify-certificate, foreign, verify-proof, federation-outcome\n\nSee README.md for usage."
        );
        return Ok(());
    }
    if command == "verify" {
        let root = match &args[1..] {
            [root] => PathBuf::from(root),
            _ => return Err("usage: temple3 verify CORPUS_ROOT".into()),
        };
        type Suite = fn(&std::path::Path) -> temple3::Result<usize>;
        let suites: [(&str, Suite); 4] = [
            ("Gate 1", vectors::kernel_vectors),
            ("Gate 2", vectors::identity_vectors),
            ("Gate 3", vectors::coordination_vectors),
            ("Gate 4", vectors::federation_vectors),
        ];
        let mut total = 0;
        for (label, suite) in suites {
            let count = suite(&root)?;
            total += count;
            println!("{label}: {count} cases verified");
        }
        println!("All {total} selected shared cases verified natively in Rust.");
        return Ok(());
    }
    if args.len() != 1 {
        return Err("JSON commands accept input on stdin only".into());
    }
    let mut bytes = vec![];
    io::stdin()
        .take(16 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 16 * 1024 * 1024 {
        return Err("input exceeds 16 MiB CLI limit".into());
    }
    let input: Value = if command == "decode" {
        canonical::decode(&bytes)?
    } else {
        serde_json::from_slice(&bytes)?
    };
    let output = match command {
        "canonical" | "decode" => {
            println!("{}", canonical::canonical_json(&input)?);
            return Ok(());
        }
        "identify" => json!(canonical::identify(
            input["kind"].as_str().ok_or("kind must be a string")?,
            &input["value"]
        )?),
        "genesis" => kernel::seal_genesis(&input)?,
        "build-delta" => kernel::build_delta(&input["temple"], &input["effects"])?,
        "transition" | "proof-transition" => {
            let r = if command == "transition" {
                kernel::evaluate_transition(&input["temple"], &input["proposal"])?
            } else {
                kernel::evaluate_proof_transition(
                    &input["temple"],
                    &input["proposal"],
                    &input["proof"],
                )?
            };
            json!({"accepted":r.accepted,"temple":r.temple,"witness":r.witness})
        }
        "verify-signature" => json!(identity::verify_envelope(
            &input["envelope"],
            input["purpose"]
                .as_str()
                .ok_or("purpose must be a string")?,
            &input["publicKey"]
        )),
        "replay" => identity::replay(&input["records"], &input["anchor"])?,
        "coordination-replay" => coordination::replay(&input["records"], &input["anchor"])?,
        "verify-certificate" => {
            let mut state = coordination::initial(&input["genesis"], &input["expectedTempleId"])?;
            for parent in input["parents"]
                .as_array()
                .ok_or("parents must be an array")?
            {
                state = coordination::advance(&state, parent)?;
            }
            coordination::advance(&state, &input["certificate"])?
        }
        "foreign" => federation::public_head(&federation::foreign_chain(
            &input["bundle"],
            &input["expectedTempleId"],
        )?)?,
        "verify-proof" => json!(federation::verify_proof(
            &input["statement"],
            &input["proof"]
        )?),
        "federation-outcome" => federation::outcome(&input["plan"], &input["bundles"])?,
        _ => return Err(format!("unknown command: {command}").into()),
    };
    println!("{}", canonical::canonical_json(&output)?);
    Ok(())
}
