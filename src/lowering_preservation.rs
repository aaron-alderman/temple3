//! Independent raw command/authority definition preservation.
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
type R<T> = Result<T, String>;
fn need(ok: bool) -> R<()> {
    if ok {
        Ok(())
    } else {
        Err("invalid-preservation".into())
    }
}
fn shape(v: &Value, keys: &str) -> R<()> {
    crate::shape(v, keys).map_err(|_| "invalid-preservation".into())
}
fn canonical(v: &Value) -> R<String> {
    crate::canonical::canonical_json(v).map_err(|_| "invalid-preservation".into())
}
fn hash(kind: &str, v: &Value) -> R<String> {
    crate::canonical::identify(kind, v).map_err(|_| "invalid-preservation".into())
}
fn string(v: &Value) -> R<&str> {
    v.as_str().ok_or_else(|| "invalid-preservation".into())
}
fn text(v: &Value) -> bool {
    v.as_str().is_some_and(|s| !s.is_empty()) && canonical(v).is_ok_and(|s| s.len() <= 1024)
}
struct Index(BTreeMap<String, BTreeMap<String, Vec<String>>>);
impl Index {
    fn values(&self, s: &str, p: &str) -> &[String] {
        self.0
            .get(s)
            .and_then(|r| r.get(p))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }
    fn one(&self, s: &str, p: &str) -> R<&str> {
        let v = self.values(s, p);
        need(v.len() == 1)?;
        Ok(&v[0])
    }
    fn fields(&self, s: &str, keys: &str) -> R<()> {
        let actual: BTreeSet<&str> = self
            .0
            .get(s)
            .map(|r| r.keys().map(String::as_str).collect())
            .unwrap_or_default();
        need(actual == keys.split_whitespace().collect())
    }
    fn eq(&self, s: &str, p: &str, v: &str) -> R<()> {
        let values = self.values(s, p);
        need(values.len() == 1 && values[0] == v)
    }
    fn ordered<'a>(&'a self, s: &str, p: &str, key: &str) -> R<Vec<&'a str>> {
        let mut result = Vec::new();
        for id in self.values(s, p) {
            result.push((self.one(id, key)?, id.as_str()));
        }
        result.sort();
        Ok(result.into_iter().map(|(_, s)| s).collect())
    }
}
pub fn inspect(x: &Value, anchor: &Value) -> R<Value> {
    canonical(x)?;
    shape(x, "obligations facts executionPolicy")?;
    let o = &x["obligations"];
    shape(
        o,
        "kind sourceGeneration command root memberPredicate targetType statePredicate from to reviewer historyPredicate historyValue evidencePredicate evidenceMax",
    )?;
    need(
        o["kind"] == "temple3.transition-lowering-obligations/v0.1"
            && o.as_object().unwrap().values().all(text),
    )?;
    for k in [
        "from",
        "to",
        "historyValue",
        "statePredicate",
        "historyPredicate",
        "evidencePredicate",
    ] {
        let v = string(&o[k])?;
        need(!v.contains('<') && !v.contains('$'))?;
    }
    let source = string(&o["sourceGeneration"])?;
    let maximum = string(&o["evidenceMax"])?;
    need(
        source.len() == 64
            && source
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            && maximum.len() <= 4
            && !maximum.starts_with('0')
            && maximum.bytes().all(|b| b.is_ascii_digit())
            && maximum
                .parse::<u32>()
                .is_ok_and(|n| (1..=4096).contains(&n))
            && o["from"] != o["to"],
    )?;
    let state = string(&o["statePredicate"])?;
    let history = string(&o["historyPredicate"])?;
    let evidence = string(&o["evidencePredicate"])?;
    need(BTreeSet::from([state, history, evidence]).len() == 3)?;
    let obligation_id = hash("lowering-obligations", o)?;
    need(anchor.as_str() == Some(&obligation_id))?;
    need(
        x["executionPolicy"]
            == json!({"compiledCommandTypes":["AuthoringMove"],"commandCompiler":"javascriptAuthoringMovePlanV1"}),
    )?;
    let facts = x["facts"].as_array().ok_or("invalid-preservation")?;
    need((1..=4096).contains(&facts.len()) && canonical(&x["facts"])?.len() <= 1048576)?;
    let mut ix = Index(BTreeMap::new());
    let mut unique = BTreeSet::new();
    for f in facts {
        shape(f, "s p o pol")?;
        need(
            text(&f["s"])
                && text(&f["p"])
                && f["o"].is_string()
                && canonical(&f["o"])?.len() <= 4096
                && f["pol"] == "is",
        )?;
        let s = string(&f["s"])?;
        let p = string(&f["p"])?;
        let v = string(&f["o"])?;
        need(unique.insert((s.to_owned(), p.to_owned(), v.to_owned())))?;
        ix.0.entry(s.into())
            .or_default()
            .entry(p.into())
            .or_default()
            .push(v.into());
    }
    let command = string(&o["command"])?;
    ix.fields(command,"isa name description order effect targetRoot memberPredicate targetType admissionSuite targetArg arg setFact guard")?;
    let mut types = ix.values(command, "isa").to_vec();
    types.sort();
    need(types == ["AuthoringMove", "Command"])?;
    for (p, v) in [
        ("effect", "setFacts"),
        ("targetRoot", string(&o["root"])?),
        ("memberPredicate", string(&o["memberPredicate"])?),
        ("targetType", string(&o["targetType"])?),
        ("targetArg", "1"),
    ] {
        ix.eq(command, p, v)?;
    }
    for p in ["name", "description", "order", "admissionSuite"] {
        ix.one(command, p)?;
    }
    let args = ix.ordered(command, "arg", "position")?;
    need(args.len() == 2)?;
    ix.fields(args[0], "isa name description position valueType required")?;
    ix.fields(
        args[1],
        "isa name description position valueType required minLength maxLength pattern",
    )?;
    for (i, a) in args.iter().enumerate() {
        ix.eq(a, "isa", "AuthoringMoveArg")?;
        ix.eq(a, "position", &(i + 1).to_string())?;
        ix.eq(a, "required", "true")?;
        ix.eq(a, "valueType", if i == 0 { "ref" } else { "string" })?;
        ix.one(a, "name")?;
        ix.one(a, "description")?;
    }
    ix.eq(args[1], "minLength", "1")?;
    ix.eq(args[1], "maxLength", maximum)?;
    ix.eq(args[1], "pattern", r"\S")?;
    let guard = ix.one(command, "guard")?;
    ix.fields(guard, "isa mode denialMessage condition")?;
    ix.eq(guard, "isa", "AuthoringMoveGuard")?;
    ix.eq(guard, "mode", "all")?;
    need(text(&json!(ix.one(guard, "denialMessage")?)))?;
    let condition = ix.one(guard, "condition")?;
    ix.fields(condition, "isa scope predicate operator valueType value")?;
    for (p, v) in [
        ("isa", "AuthoringMoveGuardCondition"),
        ("scope", "target"),
        ("predicate", state),
        ("operator", "equal"),
        ("valueType", "string"),
        ("value", string(&o["from"])?),
    ] {
        ix.eq(condition, p, v)?;
    }
    let effects = ix.ordered(command, "setFact", "order")?;
    need(effects.len() == 3)?;
    for (i, s) in effects.iter().enumerate() {
        ix.fields(s, "isa order s p o")?;
        ix.eq(s, "isa", "AuthoringMoveFact")?;
        ix.eq(s, "order", &(i + 1).to_string())?;
        ix.eq(s, "s", "$target")?;
        ix.eq(s, "p", [state, history, evidence][i])?;
        ix.eq(
            s,
            "o",
            [string(&o["to"])?, string(&o["historyValue"])?, "<2>"][i],
        )?;
    }
    need(!facts.iter().any(|f| {
        f["p"] == "role" || (f["p"] == "isa" && f["o"] == "CommandContextAuthorityPolicy")
    }))?;
    let policies: Vec<&str> =
        ix.0.keys()
            .map(String::as_str)
            .filter(|s| {
                ix.values(s, "isa")
                    .iter()
                    .any(|v| v == "ResourcePermissionPolicy")
                    && ix.values(s, "resource").iter().any(|v| v == command)
            })
            .collect();
    need(policies.len() == 1)?;
    let policy = policies[0];
    ix.fields(policy, "isa resource mode grant")?;
    ix.eq(policy, "isa", "ResourcePermissionPolicy")?;
    ix.eq(policy, "resource", command)?;
    ix.eq(policy, "mode", "scoped")?;
    let grants = ix.values(policy, "grant");
    need(grants.len() == 3)?;
    let mut permissions = Vec::new();
    for g in grants {
        ix.fields(g, "isa principal permission")?;
        ix.eq(g, "isa", "ResourcePermissionGrant")?;
        ix.eq(g, "principal", string(&o["reviewer"])?)?;
        permissions.push(ix.one(g, "permission")?);
    }
    permissions.sort();
    need(permissions == ["perm.execute", "perm.read", "perm.write"])?;
    let execution: BTreeSet<&str> = [command, guard, condition]
        .into_iter()
        .chain(args)
        .chain(effects)
        .collect();
    let authority: BTreeSet<&str> = std::iter::once(policy)
        .chain(grants.iter().map(String::as_str))
        .collect();
    let select = |subjects: &BTreeSet<&str>| -> R<Value> {
        let mut selected = Vec::new();
        for f in facts {
            if subjects.contains(string(&f["s"])?) {
                selected.push((canonical(f)?, f.clone()));
            }
        }
        selected.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
        Ok(json!(
            selected.into_iter().map(|(_, f)| f).collect::<Vec<_>>()
        ))
    };
    Ok(
        json!({"kind":"temple3.transition-lowering-inspection/v0.1","obligationId":obligation_id,"artifactId":hash("lowered-fact-artifact",&x["facts"])?,"executionId":hash("lowered-execution",&json!({"executionPolicy":x["executionPolicy"],"definitions":select(&execution)?}))?,"authorityId":hash("lowered-authority",&select(&authority)?)?,"boundary":"declared-program-definition-only"}),
    )
}
