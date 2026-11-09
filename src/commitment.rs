use crate::{
    Result, array,
    canonical::{canonical_json, identify, is_id},
    err, require, shape, string,
};
use num_bigint::BigUint;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::OnceLock,
};

pub fn cell_for(value: &Value) -> Result<Value> {
    Ok(json!({"id":identify("cell",value)?,"value":value}))
}
pub fn normalize_cells(cells: &Value) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    for cell in array(cells, "invalid-state")? {
        let value = cell.get("value").ok_or_else(|| err("invalid-cell"))?;
        let expected = cell_for(value)?;
        require(
            cell.get("id").is_none_or(|id| *id == expected["id"]),
            "forged-cell-identity",
        )?;
        out.push(expected);
    }
    out.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    require(
        !out.windows(2).any(|p| p[0]["id"] == p[1]["id"]),
        "duplicate-cell",
    )?;
    Ok(out)
}
fn node(left: &str, right: &str) -> Result<String> {
    identify("cell-tree-node", &json!({"left":left,"right":right}))
}
fn leaf(id: &str) -> Result<String> {
    identify("cell-tree-leaf", &json!({"cellId":id}))
}
fn empty() -> &'static Vec<String> {
    static HASHES: OnceLock<Vec<String>> = OnceLock::new();
    HASHES.get_or_init(|| {
        // All values here are fixed canonical protocol constants.
        let mut v = vec![identify("cell-tree-empty", &Value::Null).expect("constant")];
        for h in 0..256 {
            v.push(node(&v[h], &v[h]).expect("constant"));
        }
        v
    })
}
fn key(id: &Value) -> Result<BigUint> {
    require(is_id(id, "cell"), "invalid-cell-identity")?;
    BigUint::parse_bytes(&string(id, "invalid-cell-identity")?.as_bytes()[12..], 16)
        .ok_or_else(|| err("invalid-cell-identity"))
}
fn wrap(root: &str) -> Result<String> {
    identify("cell-set", &json!({"depth":"256","root":root}))
}
pub fn cell_set_root(cells: &Value) -> Result<String> {
    let cells = normalize_cells(cells)?;
    let mut current = BTreeMap::new();
    for c in cells {
        current.insert(
            key(&c["id"])?,
            leaf(string(&c["id"], "invalid-cell-identity")?)?,
        );
    }
    for h in 0..256 {
        let parents: BTreeSet<BigUint> = current.keys().map(|k| k >> 1).collect();
        let mut next = BTreeMap::new();
        for p in parents {
            let l = &p << 1;
            next.insert(
                p,
                node(
                    current.get(&l).unwrap_or(&empty()[h]),
                    current.get(&(l + 1u8)).unwrap_or(&empty()[h]),
                )?,
            );
        }
        current = next;
    }
    wrap(current.get(&BigUint::default()).unwrap_or(&empty()[256]))
}
pub fn state_root_from(predicate: &Value, cells: &Value) -> Result<String> {
    require(
        is_id(predicate, "predicate") && is_id(cells, "cell-set"),
        "invalid-state-commitment",
    )?;
    identify(
        "state",
        &json!({"activePredicateId":predicate,"cellSetRoot":cells}),
    )
}
pub fn verify_cell_proof(proof: &Value, expected: &Value) -> Result<bool> {
    shape(proof, "kind cellId outcome cell siblings cellSetRoot")
        .map_err(|_| err("invalid-cell-proof"))?;
    require(
        proof["kind"] == "temple3.sparse-cell-proof/v0.1",
        "invalid-cell-proof",
    )?;
    let mut position = key(&proof["cellId"])?;
    require(is_id(expected, "cell-set"), "invalid-cell-proof")?;
    if proof["cellSetRoot"] != *expected {
        return Ok(false);
    }
    let entries = array(&proof["siblings"], "invalid-cell-proof")?;
    let mut current = match proof["outcome"].as_str() {
        Some("present") => {
            shape(&proof["cell"], "id value").map_err(|_| err("invalid-cell-proof"))?;
            let c = cell_for(
                proof["cell"]
                    .get("value")
                    .ok_or_else(|| err("invalid-cell-proof"))?,
            )?;
            if c["id"] != proof["cell"]["id"] || c["id"] != proof["cellId"] {
                return Ok(false);
            }
            leaf(string(&c["id"], "invalid-cell-proof")?)?
        }
        Some("absent") => {
            require(proof["cell"].is_null(), "invalid-cell-proof")?;
            empty()[0].clone()
        }
        _ => return Err(err("invalid-cell-proof")),
    };
    let mut siblings = BTreeMap::new();
    let mut previous = None;
    for entry in entries {
        shape(entry, "height hash").map_err(|_| err("invalid-cell-proof"))?;
        let text = string(&entry["height"], "invalid-cell-proof")?;
        let h = text
            .parse::<usize>()
            .map_err(|_| err("invalid-cell-proof"))?;
        require(
            h < 256 && h.to_string() == text && previous.is_none_or(|p| h > p),
            "invalid-cell-proof",
        )?;
        require(
            is_id(
                &entry["hash"],
                if h == 0 {
                    "cell-tree-leaf"
                } else {
                    "cell-tree-node"
                },
            ) && entry["hash"] != empty()[h],
            "invalid-cell-proof",
        )?;
        siblings.insert(h, string(&entry["hash"], "invalid-cell-proof")?);
        previous = Some(h);
    }
    for h in 0..256 {
        let sibling = siblings.get(&h).copied().unwrap_or(&empty()[h]);
        current = if position.bit(0) {
            node(sibling, &current)?
        } else {
            node(&current, sibling)?
        };
        position >>= 1;
    }
    Ok(json!(wrap(&current)?) == *expected)
}

/// Authenticated sparse frontier, usable only for its exact target identities.
pub struct Multiproof {
    targets: BTreeMap<BigUint, String>,
    present: BTreeSet<String>,
    frontier: BTreeMap<(usize, BigUint), String>,
    expected: String,
    pub cells: Vec<Value>,
}
impl Multiproof {
    pub fn authenticate(proof: &Value, expected: &Value, ids: &[Value]) -> Result<Self> {
        require(
            canonical_json(proof)?.len() <= 1_048_576,
            "transition-proof-too-large",
        )?;
        let invalid = || err("invalid-transition-proof");
        require(is_id(expected, "cell-set"), "invalid-transition-proof")?;
        shape(proof, "kind cellIds cells frontier").map_err(|_| invalid())?;
        require(
            proof["kind"] == "temple3.transition-multiproof/v0.1",
            "invalid-transition-proof",
        )?;
        let mut sorted: Vec<Value> = ids.to_vec();
        sorted.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
        sorted.dedup();
        require(
            proof["cellIds"] == json!(sorted),
            "invalid-transition-proof",
        )?;
        let mut targets = BTreeMap::new();
        for id in &sorted {
            targets.insert(key(id)?, string(id, "invalid-transition-proof")?.to_owned());
        }
        let mut present = BTreeSet::new();
        let cells = array(&proof["cells"], "invalid-transition-proof")?.clone();
        let mut previous = String::new();
        for c in &cells {
            shape(c, "id value").map_err(|_| invalid())?;
            let id = string(&c["id"], "invalid-transition-proof")?;
            require(
                id > previous.as_str()
                    && sorted.contains(&c["id"])
                    && identify("cell", &c["value"])? == id,
                "invalid-transition-proof",
            )?;
            previous = id.into();
            present.insert(id.into());
        }
        let mut frontier = BTreeMap::new();
        let mut previous = None;
        for e in array(&proof["frontier"], "invalid-transition-proof")? {
            shape(e, "height position hash").map_err(|_| invalid())?;
            let htext = string(&e["height"], "invalid-transition-proof")?;
            let ptext = string(&e["position"], "invalid-transition-proof")?;
            require(
                htext.len() <= 3 && ptext.len() <= 78,
                "invalid-transition-proof",
            )?;
            let h = htext.parse::<usize>().map_err(|_| invalid())?;
            let p = crate::decimal(&e["position"]).map_err(|_| invalid())?;
            require(
                h < 256 && h.to_string() == htext,
                "invalid-transition-proof",
            )?;
            let coordinate = (h, p.clone());
            require(
                p < (BigUint::from(1u8) << (256 - h))
                    && previous.as_ref().is_none_or(|prev| coordinate > *prev)
                    && is_id(
                        &e["hash"],
                        if h == 0 {
                            "cell-tree-leaf"
                        } else {
                            "cell-tree-node"
                        },
                    )
                    && e["hash"] != empty()[h],
                "invalid-transition-proof",
            )?;
            frontier.insert(
                coordinate.clone(),
                string(&e["hash"], "invalid-transition-proof")?.into(),
            );
            previous = Some(coordinate);
        }
        require(
            !targets.is_empty() || (frontier.is_empty() && present.is_empty()),
            "invalid-transition-proof",
        )?;
        let result = Self {
            targets,
            present,
            frontier,
            expected: string(expected, "invalid-transition-proof")?.into(),
            cells,
        };
        require(
            result.root(&result.present)? == result.expected,
            "invalid-transition-proof",
        )?;
        Ok(result)
    }
    fn root(&self, present: &BTreeSet<String>) -> Result<String> {
        if self.targets.is_empty() {
            return Ok(self.expected.clone());
        }
        fn descend(
            m: &Multiproof,
            h: usize,
            pos: BigUint,
            keys: &[&BigUint],
            present: &BTreeSet<String>,
            used: &mut BTreeSet<(usize, BigUint)>,
        ) -> Result<String> {
            if keys.is_empty() {
                let coord = (h, pos);
                if let Some(hash) = m.frontier.get(&coord) {
                    used.insert(coord);
                    return Ok(hash.clone());
                }
                return Ok(empty()[h].clone());
            }
            if h == 0 {
                let id = &m.targets[keys[0]];
                return if present.contains(id) {
                    leaf(id)
                } else {
                    Ok(empty()[0].clone())
                };
            }
            let (right, left): (Vec<_>, Vec<_>) =
                keys.iter().copied().partition(|k| k.bit((h - 1) as u64));
            node(
                &descend(m, h - 1, &pos << 1, &left, present, used)?,
                &descend(m, h - 1, (&pos << 1) + 1u8, &right, present, used)?,
            )
        }
        let mut used = BTreeSet::new();
        let raw = descend(
            self,
            256,
            BigUint::default(),
            &self.targets.keys().collect::<Vec<_>>(),
            present,
            &mut used,
        )?;
        require(
            used == self.frontier.keys().cloned().collect(),
            "invalid-transition-proof",
        )?;
        wrap(&raw)
    }
    pub fn successor(&self, consume: &Value, create: &Value) -> Result<String> {
        let mut updated = self.present.clone();
        for id in array(consume, "invalid-transition-proof")? {
            require(
                updated.remove(string(id, "invalid-transition-proof")?),
                "invalid-transition-proof",
            )?;
        }
        for c in array(create, "invalid-transition-proof")? {
            let id = string(&c["id"], "invalid-transition-proof")?;
            require(
                self.targets.values().any(|v| v == id) && updated.insert(id.into()),
                "invalid-transition-proof",
            )?;
        }
        self.root(&updated)
    }
}
