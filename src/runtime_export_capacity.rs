//! Pure capacity computation. A passing result never grants source or signer authority.
use serde_json::{Value, json};
use std::collections::{BTreeSet, HashSet};
type R<T> = Result<T, String>;
const MAX_SAFE: u128 = 9_007_199_254_740_991;

fn fail<T>(code: &str) -> R<T> {
    Err(code.into())
}
fn exact(value: &Value, fields: &str, code: &str) -> R<()> {
    let Some(object) = value.as_object() else {
        return fail(code);
    };
    let actual: BTreeSet<&str> = object.keys().map(String::as_str).collect();
    let expected: BTreeSet<&str> = fields.split_whitespace().collect();
    if actual == expected {
        Ok(())
    } else {
        fail(code)
    }
}
fn decimal(value: &Value, positive: bool, code: &str) -> R<u128> {
    let Some(source) = value.as_str() else {
        return fail(code);
    };
    if source.is_empty()
        || !source.bytes().all(|b| b.is_ascii_digit())
        || (source.len() > 1 && source.starts_with('0'))
    {
        return fail(code);
    }
    let number = source.parse::<u128>().map_err(|_| code.to_string())?;
    if number > MAX_SAFE || positive && number == 0 {
        return fail(code);
    }
    Ok(number)
}
fn bytes(value: &Value) -> R<u128> {
    crate::canonical::canonical_json(value)
        .map(|s| s.len() as u128)
        .map_err(|_| "invalid-export-capacity-record".into())
}
fn multiply(a: u128, b: u128) -> R<u128> {
    a.checked_mul(b)
        .ok_or_else(|| "invalid-export-capacity-profile".into())
}

struct Limits {
    events: u128,
    seal: u128,
    event: u128,
    receipt: u128,
    record: u128,
    ack: u128,
    journal: u128,
    pending: u128,
    delivery_records: u128,
    delivery_bytes: u128,
}
fn profile(value: &Value) -> R<Limits> {
    exact(
        value,
        "kind maxEvents maxSealBytes maxEventBytes maxReceiptBytes maxExportRecordBytes maxAckRecordBytes maxJournalBytes maxPendingBytes maxDeliveryRecords maxDeliveryBytes overflow retention forgiveness",
        "invalid-export-capacity-shape",
    )?;
    if value["kind"] != "temple3.runtime-export-capacity-profile/v0.1" {
        return fail("invalid-export-capacity-profile");
    }
    let n = Limits {
        events: decimal(&value["maxEvents"], true, "invalid-export-capacity-profile")?,
        seal: decimal(
            &value["maxSealBytes"],
            true,
            "invalid-export-capacity-profile",
        )?,
        event: decimal(
            &value["maxEventBytes"],
            true,
            "invalid-export-capacity-profile",
        )?,
        receipt: decimal(
            &value["maxReceiptBytes"],
            true,
            "invalid-export-capacity-profile",
        )?,
        record: decimal(
            &value["maxExportRecordBytes"],
            true,
            "invalid-export-capacity-profile",
        )?,
        ack: decimal(
            &value["maxAckRecordBytes"],
            true,
            "invalid-export-capacity-profile",
        )?,
        journal: decimal(
            &value["maxJournalBytes"],
            true,
            "invalid-export-capacity-profile",
        )?,
        pending: decimal(
            &value["maxPendingBytes"],
            true,
            "invalid-export-capacity-profile",
        )?,
        delivery_records: decimal(
            &value["maxDeliveryRecords"],
            true,
            "invalid-export-capacity-profile",
        )?,
        delivery_bytes: decimal(
            &value["maxDeliveryBytes"],
            true,
            "invalid-export-capacity-profile",
        )?,
    };
    let invalid = n
        .seal
        .checked_add(n.event)
        .and_then(|v| v.checked_add(n.receipt))
        .is_none_or(|v| v > n.record)
        || multiply(
            n.events,
            n.record
                .checked_add(n.ack)
                .ok_or("invalid-export-capacity-profile")?,
        )? > n.journal
        || multiply(n.events, n.record)? > n.pending
        || n.delivery_records > n.events
        || multiply(n.delivery_records, n.record)? > n.delivery_bytes
        || value["overflow"] != "refuse-before-source-command"
        || value["retention"] != "retain-all-no-pruning"
        || value["forgiveness"]
            != json!({"disposition":"refuse","scope":"runtime-export-capacity"});
    if invalid {
        return fail("invalid-export-capacity-profile");
    }
    Ok(n)
}
fn reserve(value: &Value) -> R<Value> {
    let p = &value["profile"];
    let n = profile(p)?;
    let state = &value["state"];
    exact(
        state,
        "eventCount acknowledgedCount",
        "invalid-export-capacity-state",
    )?;
    let events = decimal(&state["eventCount"], false, "invalid-export-capacity-state")?;
    let acknowledged = decimal(
        &state["acknowledgedCount"],
        false,
        "invalid-export-capacity-state",
    )?;
    if acknowledged > events {
        return fail("invalid-export-capacity-state");
    }
    let slot = n.record + n.ack;
    let after_events = events + 1;
    let after_pending = events - acknowledged + 1;
    let after_journal = after_events * slot;
    if after_events > n.events {
        return fail("export-event-capacity");
    }
    if after_pending * n.record > n.pending {
        return fail("export-pending-capacity");
    }
    if after_journal > n.journal {
        return fail("export-journal-capacity");
    }
    Ok(
        json!({"status":"reserved","overflow":p["overflow"],"retention":p["retention"],"before":{"events":events.to_string(),"pending":(events-acknowledged).to_string(),"journalReserved":(events*slot).to_string()},"after":{"events":after_events.to_string(),"pending":after_pending.to_string(),"journalReserved":after_journal.to_string()}}),
    )
}
fn measure_record_for(p: &Value, record: &Value) -> R<(Value, String)> {
    let n = profile(p)?;
    exact(
        record,
        "kind sealJson event receipt",
        "invalid-export-capacity-record",
    )?;
    if record["kind"] != "sourcery-shadow-export" {
        return fail("invalid-export-capacity-record");
    }
    let Some(seal_source) = record["sealJson"].as_str() else {
        return fail("invalid-export-capacity-record");
    };
    let seal: Value = serde_json::from_str(seal_source)
        .map_err(|_| "invalid-export-capacity-record".to_string())?;
    let Some(id) = seal["sealId"].as_str() else {
        return fail("invalid-export-capacity-record");
    };
    if id.len() != 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return fail("invalid-export-capacity-record");
    }
    let seal_bytes = seal_source.len() as u128;
    let event_bytes = bytes(&record["event"])?;
    let receipt_bytes = bytes(&record["receipt"])?;
    let record_bytes = bytes(record)?;
    if seal_bytes > n.seal {
        return fail("export-seal-capacity");
    }
    if event_bytes > n.event {
        return fail("export-event-byte-capacity");
    }
    if receipt_bytes > n.receipt {
        return fail("export-receipt-capacity");
    }
    if record_bytes > n.record {
        return fail("export-record-capacity");
    }
    Ok((
        json!({"sourceSealId":id,"sealBytes":seal_bytes.to_string(),"eventBytes":event_bytes.to_string(),"receiptBytes":receipt_bytes.to_string(),"recordBytes":record_bytes.to_string()}),
        id.into(),
    ))
}
fn measure_ack(value: &Value) -> R<Value> {
    let n = profile(&value["profile"])?;
    let ack = &value["ack"];
    exact(
        ack,
        "kind sourceSealId receipt",
        "invalid-export-capacity-ack",
    )?;
    let Some(id) = ack["sourceSealId"].as_str() else {
        return fail("invalid-export-capacity-ack");
    };
    if ack["kind"] != "sourcery-shadow-export-ack"
        || id.len() != 64
        || !id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return fail("invalid-export-capacity-ack");
    }
    let size = bytes(ack)?;
    if size > n.ack {
        return fail("export-ack-capacity");
    }
    Ok(json!({"sourceSealId":id,"recordBytes":size.to_string()}))
}
fn delivery(value: &Value) -> R<Value> {
    let p = &value["profile"];
    let n = profile(p)?;
    let Some(pending) = value["pending"].as_array() else {
        return fail("invalid-export-delivery");
    };
    let Some(selection) = value["selection"].as_array() else {
        return fail("invalid-export-delivery");
    };
    if selection.is_empty() || selection.len() as u128 > n.delivery_records {
        return fail("export-delivery-count");
    }
    let mut records: Vec<(&Value, String)> = Vec::new();
    for record in pending {
        let (_, id) = measure_record_for(p, record)?;
        records.push((record, id));
    }
    let mut seen = HashSet::new();
    let mut bundles = Vec::new();
    let mut ids = Vec::new();
    for selected in selection {
        let Some(id) = selected.as_str() else {
            return fail("invalid-export-delivery");
        };
        if id.len() != 64
            || !id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || !seen.insert(id)
        {
            return fail("invalid-export-delivery");
        }
        let Some((record, _)) = records.iter().find(|(_, candidate)| candidate == id) else {
            return fail("unknown-pending-export");
        };
        let seal: Value = serde_json::from_str(record["sealJson"].as_str().unwrap()).unwrap();
        bundles.push(json!({"seal":seal,"event":record["event"]}));
        ids.push(Value::String(id.into()));
    }
    let bundles_value = Value::Array(bundles);
    let size = bytes(&bundles_value)?;
    if size > n.delivery_bytes {
        return fail("export-delivery-bytes");
    }
    Ok(
        json!({"status":"ready","sourceSealIds":ids,"bytes":size.to_string(),"bundles":bundles_value}),
    )
}
pub fn evaluate(value: &Value) -> R<Value> {
    let Some(op) = value.get("op").and_then(Value::as_str) else {
        return fail("invalid-export-capacity-operation");
    };
    match op {
        "reserve" => reserve(value),
        "record" => measure_record_for(&value["profile"], &value["record"]).map(|v| v.0),
        "ack" => measure_ack(value),
        "delivery" => delivery(value),
        _ => fail("invalid-export-capacity-operation"),
    }
}
