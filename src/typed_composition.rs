//! Atomic native composition with recomputed dependency and authority footprints.
use serde_json::{Value, json};
use std::collections::BTreeSet;
type R<T> = Result<T, String>;
const KIND: &str = "temple3.typed-composition-state/v0.1";
pub const REPRESENTATIVE_KIND: &str = "temple3.representative-composition-state/v0.1";
fn need(b: bool, c: &str) -> R<()> {
    if b { Ok(()) } else { Err(c.into()) }
}
fn shape(v: &Value, k: &str, c: &str) -> R<()> {
    crate::shape(v, k).map_err(|_| c.into())
}
fn canonical(v: &Value, c: &str) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|_| c.into())
}
fn id(k: &str, v: &Value) -> R<String> {
    crate::canonical::identify(k, v).map_err(|e| e.code)
}
fn actor(v: &Value) -> bool {
    v.as_str().is_some_and(|s| {
        !s.is_empty()
            && s.len() <= 64
            && s.as_bytes()[0].is_ascii_lowercase()
            && s.bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'.' || b == b'-')
    })
}
fn reference(v: &Value, p: &str) -> bool {
    v.as_str().and_then(|s| s.strip_prefix(p)).is_some_and(|s| {
        !s.is_empty() && s.as_bytes()[0] != b'0' && s.bytes().all(|b| b.is_ascii_digit())
    })
}
fn native_validate(c: &Value, s: &Value, kind: &str) -> R<()> {
    match (kind, c.as_str()) {
        (_, Some("basis-decision-command-v1")) => crate::basis_decision::validate(s),
        (_, Some("basis-consequence-command-v1")) => crate::basis_consequence::validate(s),
        (REPRESENTATIVE_KIND, Some("inventory-command-v1")) => crate::inventory::validate(s),
        _ => Err("unsupported-component".into()),
    }
}
fn native_formation(c: &Value, f: &Value, kind: &str) -> R<Value> {
    match (kind, c.as_str()) {
        (_, Some("basis-decision-command-v1")) => crate::basis_decision::formation(f),
        (_, Some("basis-consequence-command-v1")) => crate::basis_consequence::formation(f),
        (REPRESENTATIVE_KIND, Some("inventory-command-v1")) => crate::inventory::formation(f),
        _ => Err("unsupported-component".into()),
    }
}
fn native_next(c: &Value, s: &Value, a: &Value, cmd: &Value, kind: &str) -> R<Value> {
    match (kind, c.as_str()) {
        (_, Some("basis-decision-command-v1")) => crate::basis_decision::next_state(s, a, cmd),
        (_, Some("basis-consequence-command-v1")) => {
            crate::basis_consequence::next_state(s, a, cmd)
        }
        (REPRESENTATIVE_KIND, Some("inventory-command-v1")) => {
            crate::inventory::next_state(s, a, cmd)
        }
        _ => Err("unsupported-component".into()),
    }
}
fn lookup(s: &Value, field: &str, key: &Value) -> R<usize> {
    s[field]
        .as_array()
        .ok_or("invalid-state")?
        .iter()
        .position(|c| c["id"] == *key)
        .ok_or("unknown-reference".into())
}
pub fn generation(s: &Value, key: &Value) -> R<String> {
    let c = &s["components"][lookup(s, "components", key)?];
    let imports: Vec<Value>=s["links"].as_array().ok_or("invalid-state")?.iter().filter(|l|l["consumerId"]==*key).map(|l|json!({"producerId":l["producerId"],"exportName":l["exportName"],"outputKind":l["outputKind"],"generation":l["generation"]})).collect();
    id(
        "component-generation",
        &json!({"componentId":key,"contract":c["contract"],"state":c["state"],"imports":imports}),
    )
}
fn export_for(c: &Value, name: &Value, kind: &str) -> R<Value> {
    native_validate(&c["contract"], &c["state"], kind)?;
    match name.as_str() {
        Some("records") if c["contract"] != "inventory-command-v1" => Ok(
            json!({"kind":"record-identities/v1","value":c["state"]["records"].as_array().unwrap().iter().map(|r|r["id"].clone()).collect::<Vec<_>>()}),
        ),
        Some("decisions") if c["contract"] != "inventory-command-v1" => {
            Ok(json!({"kind":"decision-witnesses/v1","value":c["state"]["decisions"]}))
        }
        Some("restrictions") if c["contract"] == "basis-consequence-command-v1" => Ok(
            json!({"kind":"restriction-history/v1","value":{"restrictions":c["state"]["restrictions"],"releases":c["state"]["releases"]}}),
        ),
        Some("balances") if c["contract"] == "inventory-command-v1" => {
            Ok(json!({"kind":"inventory-balances/v1","value":c["state"]["balances"]}))
        }
        Some("transfers") if c["contract"] == "inventory-command-v1" => {
            Ok(json!({"kind":"inventory-transfers/v1","value":c["state"]["transfers"]}))
        }
        _ => Err("unknown-export".into()),
    }
}
pub fn export(c: &Value, name: &Value) -> R<Value> {
    export_for(c, name, KIND)
}
pub fn representative_export(c: &Value, name: &Value) -> R<Value> {
    export_for(c, name, REPRESENTATIVE_KIND)
}
fn visit(
    s: &Value,
    key: &Value,
    visiting: &mut BTreeSet<String>,
    done: &mut BTreeSet<String>,
) -> R<()> {
    let k = key.as_str().ok_or("invalid-state")?.to_owned();
    need(!visiting.contains(&k), "dependency-cycle")?;
    if done.contains(&k) {
        return Ok(());
    }
    visiting.insert(k.clone());
    for l in s["links"].as_array().unwrap() {
        if l["consumerId"] == *key {
            visit(s, &l["producerId"], visiting, done)?;
        }
    }
    visiting.remove(&k);
    done.insert(k);
    Ok(())
}
fn graph(s: &Value) -> R<()> {
    let mut edges = BTreeSet::new();
    for l in s["links"].as_array().unwrap() {
        shape(
            l,
            "id consumerId producerId exportName outputKind generation",
            "invalid-state",
        )?;
        let consumer = lookup(s, "components", &l["consumerId"])?;
        let producer = lookup(s, "components", &l["producerId"])?;
        need(consumer != producer, "dependency-cycle")?;
        need(
            edges.insert(canonical(
                &json!([l["consumerId"], l["producerId"], l["exportName"]]),
                "invalid-state",
            )?),
            "duplicate-link",
        )?;
        need(
            export_for(
                &s["components"][producer],
                &l["exportName"],
                s["kind"].as_str().ok_or("invalid-state")?,
            )?["kind"]
                == l["outputKind"],
            "wrong-export-kind",
        )?;
    }
    let mut visiting = BTreeSet::new();
    let mut done = BTreeSet::new();
    for c in s["components"].as_array().unwrap() {
        visit(s, &c["id"], &mut visiting, &mut done)?;
    }
    for l in s["links"].as_array().unwrap() {
        need(
            l["generation"] == generation(s, &l["producerId"])?,
            "stale-generation",
        )?;
    }
    Ok(())
}
fn validate_as(s: &Value, expected_kind: &str) -> R<()> {
    let code = "invalid-state";
    let raw = canonical(s, code)?;
    shape(
        s,
        "kind instance participants policy sequence components links",
        code,
    )?;
    need(
        s["kind"] == expected_kind
            && s["instance"].as_str().is_some_and(|x| {
                x.len() == 32
                    && x.bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            }),
        code,
    )?;
    let people = s["participants"].as_array().ok_or(code)?;
    need(
        !people.is_empty()
            && people.len() <= 8
            && people.iter().all(actor)
            && people.windows(2).all(|w| w[0].as_str() < w[1].as_str()),
        code,
    )?;
    let p = &s["policy"];
    let representative = expected_kind == REPRESENTATIVE_KIND;
    shape(
        p,
        if representative {
            "steward forgiveness retention dependency bulk amendment retry rebind semantics sourceAuthority migration"
        } else {
            "steward forgiveness retention dependency bulk amendment retry rebind"
        },
        code,
    )?;
    let expected_policy = if representative {
        json!({"steward":p["steward"],"forgiveness":{"disposition":"refuse","scope":"representative-composition-history"},"retention":"retain-all-transition-evidence","dependency":"exact-generation","bulk":"atomic","amendment":"fixed-policy","retry":"reject-stale-basis","rebind":"steward-or-affected-native-transition-actor","semantics":"links-bind-freshness-not-command-authority","sourceAuthority":"target-only-no-source-equivalence","migration":"not-started"})
    } else {
        json!({"steward":p["steward"],"forgiveness":{"disposition":"refuse","scope":"composition-history"},"retention":"retain-all-transition-evidence","dependency":"exact-generation","bulk":"atomic","amendment":"fixed-policy","retry":"reject-stale-basis","rebind":"steward-or-affected-native-transition-actor"})
    };
    need(
        people.contains(&p["steward"]) && *p == expected_policy,
        code,
    )?;
    let seq = s["sequence"].as_str().ok_or(code)?;
    need(
        seq == "0"
            || (!seq.is_empty()
                && seq.len() <= 4
                && seq.as_bytes()[0] != b'0'
                && seq.bytes().all(|b| b.is_ascii_digit())),
        code,
    )?;
    need(seq.parse::<u32>().map_err(|_| code)? <= 1024, code)?;
    for (field, prefix, limit) in [("components", "component", 4), ("links", "link", 8)] {
        let xs = s[field].as_array().ok_or("exhausted")?;
        need(xs.len() <= limit, "exhausted")?;
        for (i, x) in xs.iter().enumerate() {
            need(
                x.is_object() && x["id"] == format!("{prefix}.{}", i + 1),
                code,
            )?;
        }
    }
    let mut instances = BTreeSet::new();
    for c in s["components"].as_array().unwrap() {
        shape(c, "id contract state", code)?;
        native_validate(&c["contract"], &c["state"], expected_kind)?;
        need(
            c["state"]["participants"] == s["participants"]
                && instances.insert(c["state"]["instance"].as_str().unwrap()),
            code,
        )?;
    }
    graph(s)?;
    need(raw.len() <= 16384, "exhausted")
}
pub fn validate(s: &Value) -> R<()> {
    validate_as(s, KIND)
}
pub fn validate_representative(s: &Value) -> R<()> {
    validate_as(s, REPRESENTATIVE_KIND)
}
pub fn formation(f: &Value) -> R<Value> {
    canonical(f, "invalid-state")?;
    shape(f, "kind instance participants policy", "invalid-state")?;
    need(
        f["kind"] == "temple3.typed-composition-formation/v0.1",
        "invalid-state",
    )?;
    let s = json!({"kind":KIND,"instance":f["instance"],"participants":f["participants"],"policy":f["policy"],"sequence":"0","components":[],"links":[]});
    validate(&s)?;
    Ok(s)
}
pub fn representative_formation(f: &Value) -> R<Value> {
    canonical(f, "invalid-state")?;
    shape(f, "kind instance participants policy", "invalid-state")?;
    need(
        f["kind"] == "temple3.representative-composition-formation/v0.1",
        "invalid-state",
    )?;
    let s = json!({"kind":REPRESENTATIVE_KIND,"instance":f["instance"],"participants":f["participants"],"policy":f["policy"],"sequence":"0","components":[],"links":[]});
    validate_as(&s, REPRESENTATIVE_KIND)?;
    Ok(s)
}
fn affected(s: &Value, seeds: &BTreeSet<String>) -> BTreeSet<String> {
    let mut result = seeds.clone();
    loop {
        let count = result.len();
        for l in s["links"].as_array().unwrap() {
            if result.contains(l["producerId"].as_str().unwrap()) {
                result.insert(l["consumerId"].as_str().unwrap().into());
            }
        }
        if result.len() == count {
            return result;
        }
    }
}
fn execute_as(s: &Value, a: &Value, command: &Value, expected_kind: &str) -> R<Value> {
    validate_as(s, expected_kind)?;
    canonical(command, "invalid-command")?;
    shape(command, "op args", "invalid-command")?;
    shape(&command["args"], "operations", "invalid-command")?;
    let ops = command["args"]["operations"]
        .as_array()
        .ok_or("invalid-command")?;
    need(
        command["op"] == "atomic" && !ops.is_empty() && ops.len() <= 8,
        "invalid-command",
    )?;
    for o in ops {
        shape(o, "op args", "invalid-command")?;
        let fields = match o["op"].as_str() {
            Some("add-component") => "contract formation",
            Some("component-command") => "componentId command",
            Some("link") => "consumerId producerId exportName outputKind generation",
            Some("rebind") => "linkId generation",
            _ => return Err("invalid-command".into()),
        };
        shape(&o["args"], fields, "invalid-command")?;
        for k in ["componentId", "consumerId", "producerId", "linkId"] {
            if let Some(v) = o["args"].get(k) {
                need(
                    reference(v, if k == "linkId" { "link." } else { "component." }),
                    "invalid-command",
                )?;
            }
        }
    }
    need(actor(a), "invalid-command")?;
    let sequence = s["sequence"].as_str().unwrap().parse::<u32>().unwrap();
    need(sequence < 1024, "exhausted")?;
    need(
        s["participants"].as_array().unwrap().contains(a),
        "unauthorized",
    )?;
    let steward = *a == s["policy"]["steward"];
    let mut n = s.clone();
    let mut changed = BTreeSet::new();
    let mut reads = BTreeSet::<String>::new();
    let mut writes = BTreeSet::<String>::new();
    let mut link_writes = BTreeSet::<String>::new();
    let mut authority = Vec::new();
    for (index, o) in ops.iter().enumerate() {
        let args = &o["args"];
        let scope: String;
        let mut rule = "composition-steward".to_owned();
        match o["op"].as_str().unwrap() {
            "add-component" => {
                need(steward, "unauthorized")?;
                need(n["components"].as_array().unwrap().len() < 4, "exhausted")?;
                let state = native_formation(&args["contract"], &args["formation"], expected_kind)?;
                need(
                    state["participants"] == s["participants"],
                    "invalid-component-participants",
                )?;
                scope = format!(
                    "component.{}",
                    n["components"].as_array().unwrap().len() + 1
                );
                n["components"]
                    .as_array_mut()
                    .unwrap()
                    .push(json!({"id":scope,"contract":args["contract"],"state":state}));
                writes.insert(scope.clone());
            }
            "component-command" => {
                let i = lookup(&n, "components", &args["componentId"])?;
                let c = &mut n["components"][i];
                c["state"] = native_next(
                    &c["contract"],
                    &c["state"],
                    a,
                    &args["command"],
                    expected_kind,
                )?;
                scope = c["id"].as_str().unwrap().into();
                changed.insert(scope.clone());
                reads.insert(scope.clone());
                writes.insert(scope.clone());
                rule = format!("native:{}", c["contract"].as_str().unwrap());
            }
            "link" => {
                need(steward, "unauthorized")?;
                need(n["links"].as_array().unwrap().len() < 8, "exhausted")?;
                scope = format!("link.{}", n["links"].as_array().unwrap().len() + 1);
                let mut l = args.clone();
                l["id"] = json!(scope);
                n["links"].as_array_mut().unwrap().push(l);
                link_writes.insert(scope.clone());
                reads.insert(args["consumerId"].as_str().unwrap().into());
                reads.insert(args["producerId"].as_str().unwrap().into());
                writes.insert(args["consumerId"].as_str().unwrap().into());
            }
            _ => {
                let i = lookup(&n, "links", &args["linkId"])?;
                need(
                    steward
                        || affected(&n, &changed)
                            .contains(n["links"][i]["producerId"].as_str().unwrap()),
                    "unauthorized",
                )?;
                let l = &mut n["links"][i];
                need(args["generation"] != l["generation"], "unchanged-value")?;
                l["generation"] = args["generation"].clone();
                scope = l["id"].as_str().unwrap().into();
                reads.insert(l["consumerId"].as_str().unwrap().into());
                reads.insert(l["producerId"].as_str().unwrap().into());
                writes.insert(l["consumerId"].as_str().unwrap().into());
                link_writes.insert(scope.clone());
                if !steward {
                    rule = "affected-native-transition-actor".into();
                }
            }
        }
        authority
            .push(json!({"operationIndex":index.to_string(),"actor":a,"scope":scope,"rule":rule}));
    }
    n["sequence"] = json!((sequence + 1).to_string());
    validate_as(&n, expected_kind)?;
    loop {
        let count = reads.len();
        for l in n["links"].as_array().unwrap() {
            if reads.contains(l["consumerId"].as_str().unwrap()) {
                reads.insert(l["producerId"].as_str().unwrap().into());
            }
        }
        if reads.len() == count {
            break;
        }
    }
    Ok(
        json!({"after":n,"footprint":{"kind":"temple3.composition-footprint/v0.1","basis":id("composition-basis",s)?,"actor":a,"validationScope":"entire-composition","componentReads":reads,"componentWrites":writes,"linkWrites":link_writes,"authorityChecks":authority}}),
    )
}
pub fn execute(s: &Value, a: &Value, command: &Value) -> R<Value> {
    execute_as(s, a, command, KIND)
}
pub fn execute_representative(s: &Value, a: &Value, command: &Value) -> R<Value> {
    execute_as(s, a, command, REPRESENTATIVE_KIND)
}
pub fn verify(i: &Value, e: &Value) -> bool {
    (|| -> R<bool> {
        canonical(i, "invalid-command")?;
        canonical(e, "invalid-command")?;
        shape(i, "before actor command footprint after", "invalid-command")?;
        shape(e, "", "invalid-command")?;
        let result = execute(&i["before"], &i["actor"], &i["command"])?;
        Ok(result["after"] == i["after"] && result["footprint"] == i["footprint"])
    })()
    .unwrap_or(false)
}
pub fn verify_representative(i: &Value, e: &Value) -> bool {
    (|| -> R<bool> {
        canonical(i, "invalid-command")?;
        canonical(e, "invalid-command")?;
        shape(i, "before actor command footprint after", "invalid-command")?;
        shape(e, "", "invalid-command")?;
        let result = execute_as(
            &i["before"],
            &i["actor"],
            &i["command"],
            REPRESENTATIVE_KIND,
        )?;
        Ok(result["after"] == i["after"] && result["footprint"] == i["footprint"])
    })()
    .unwrap_or(false)
}
