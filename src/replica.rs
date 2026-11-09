use crate::{
    Result, array, coordination as c, err, identity,
    journal::{FileJournal, LockedJournal},
    require, without,
};
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap};

pub struct Replica {
    pub journal: FileJournal,
    expected: Value,
    voter: String,
    seed: [u8; 32],
    validated: RefCell<Option<(Vec<Value>, Value)>>,
}
impl Replica {
    pub fn open(
        journal: FileJournal,
        expected: Value,
        voter: String,
        seed: [u8; 32],
        expected_tip: Option<&Value>,
    ) -> Result<Self> {
        let replica = Self {
            journal,
            expected,
            voter,
            seed,
            validated: RefCell::new(None),
        };
        let s = replica.state(expected_tip)?;
        replica.local_key(&s)?;
        Ok(replica)
    }
    pub fn create(
        journal: FileJournal,
        genesis: &Value,
        expected: Value,
        voter: String,
        seed: [u8; 32],
    ) -> Result<Self> {
        let replica = Self {
            journal,
            expected,
            voter,
            seed,
            validated: RefCell::new(None),
        };
        let s = c::initial(genesis, &replica.expected)?;
        replica.local_key(&s)?;
        replica.journal.append(genesis, &json!("empty"))?;
        Ok(replica)
    }
    fn local_key(&self, state: &Value) -> Result<()> {
        let public = hex::encode(
            ed25519_dalek::SigningKey::from_bytes(&self.seed)
                .verifying_key()
                .as_bytes(),
        );
        require(
            array(&state["profile"]["members"], "invalid-coordination-profile")?
                .iter()
                .any(|m| m["voter"] == self.voter && m["publicKey"] == public),
            "wrong-voter-key",
        )
    }
    // Cache is private and process-local. Every access decodes and verifies the
    // whole durable chain; only an identical validated prefix avoids replay.
    fn state_from_records(&self, records: Vec<Value>, tip: Option<&Value>) -> Result<Value> {
        let actual_tip = records
            .last()
            .map_or(json!("empty"), |r| r["recordId"].clone());
        require(tip.is_none_or(|t| *t == actual_tip), "journal-tip-mismatch")?;
        let cached = self.validated.borrow();
        let next = if let Some((prefix, state)) =
            cached.as_ref().filter(|(p, _)| records.starts_with(p))
        {
            c::apply_entries(state.clone(), &records[prefix.len()..], actual_tip)?
        } else {
            c::replay_state(&json!(records), &json!({"expectedTempleId":self.expected}))?
        };
        drop(cached);
        *self.validated.borrow_mut() = Some((records, next.clone()));
        Ok(next)
    }
    pub fn state(&self, tip: Option<&Value>) -> Result<Value> {
        self.state_from_records(self.journal.read()?, tip)
    }
    fn locked_state(&self, j: &LockedJournal<'_>) -> Result<Value> {
        self.state_from_records(j.read()?, None)
    }
    fn append(&self, j: &mut LockedJournal<'_>, s: &Value, entry: &Value) -> Result<Value> {
        let mut records = j.read()?;
        let record = identity::record_for(entry, records.len(), &s["tip"])?;
        let next = c::apply_entries(
            s.clone(),
            std::slice::from_ref(&record),
            record["recordId"].clone(),
        )?;
        j.append(entry, &s["tip"])?;
        records.push(record);
        *self.validated.borrow_mut() = Some((records, next.clone()));
        Ok(next)
    }
    pub fn cast_vote(&self, proposal: &Value) -> Result<Value> {
        self.cast_vote_observed(proposal, |_| Ok(()))
    }
    pub fn cast_vote_observed(
        &self,
        proposal: &Value,
        mut observe: impl FnMut(&str) -> Result<()>,
    ) -> Result<Value> {
        self.journal.transaction(|j| {
            let s = self.locked_state(j)?;
            self.local_key(&s)?;
            require(s["halted"] != true, "halted-coordination")?;
            require(
                !array(&s["equivocations"], "invalid-state")?
                    .iter()
                    .any(|e| e["voter"] == self.voter),
                "voter-equivocated",
            )?;
            let candidate = c::candidate(&s, proposal)?;
            if let Some(old) = array(&s["votes"], "invalid-state")?.iter().find(|e| {
                e["vote"]["payload"]["voter"] == self.voter
                    && e["vote"]["payload"]["height"] == candidate["decision"]["height"]
            }) {
                require(
                    without(&old["vote"]["payload"], "voter")? == candidate["decision"],
                    "vote-locked",
                )?;
                return Ok(old["vote"].clone());
            }
            let mut payload = candidate["decision"].clone();
            payload["voter"] = json!(self.voter);
            let vote = identity::sign_envelope("coordination-vote", &payload, &self.seed)?;
            let entry = json!({"kind":"coordination-vote","proposal":proposal,"vote":vote});
            j.append_observed(&entry, &s["tip"], &mut observe)?;
            Ok(vote)
        })
    }
    pub fn receive_vote(&self, proposal: &Value, vote: &Value) -> Result<Value> {
        self.journal.transaction(|j| {
            let s = self.locked_state(j)?;
            let candidate = c::candidate(
                &c::basis(&s, &vote["payload"]["parentFinalityId"])?,
                proposal,
            )?;
            c::check_vote(&s["profile"], &candidate["decision"], vote)?;
            if array(&s["votes"], "invalid-state")?
                .iter()
                .any(|v| v["vote"]["payload"] == vote["payload"])
            {
                return Ok(json!({"status":"replay"}));
            }
            self.append(
                j,
                &s,
                &json!({"kind":"coordination-vote","proposal":proposal,"vote":vote}),
            )?;
            Ok(json!({"status":"recorded"}))
        })
    }
    pub fn receive_certificate(&self, certificate: &Value) -> Result<Value> {
        self.journal.transaction(|j|{
        let s=self.locked_state(j)?;
        c::verify_certificate(&c::basis(&s,&certificate["decision"]["parentFinalityId"])?,certificate)?;
        if array(&s["history"],"invalid-state")?.iter().any(|h|h["finalityId"]==certificate["finalityId"]){return Ok(json!({"status":"replay"}));}
        let after=self.append(j,&s,&json!({"kind":"coordination-finality","certificate":certificate}))?;
        Ok(json!({"status":if after["halted"]==true{"halted"}else{"finalized"},"certificate":certificate}))
    })
    }
    /// Check past quorums before advancing the current head; never merge branches.
    pub fn resolve(&self) -> Result<Value> {
        self.journal.transaction(|j|{
        let s=self.locked_state(j)?;require(s["halted"]!=true,"halted-coordination")?;
        for head in array(&s["history"],"invalid-state")? {

            let mut groups:BTreeMap<String,(Value,Vec<Value>)>=BTreeMap::new();
            for observation in array(&s["votes"],"invalid-state")?.iter().filter(|v|v["vote"]["payload"]["parentFinalityId"]==head["finalityId"]) {
                let decision=without(&observation["vote"]["payload"],"voter")?;
                let id=crate::canonical::identify("finality",&decision)?;
                groups.entry(id).or_insert_with(||(observation["proposal"].clone(),vec![])).1.push(observation["vote"].clone());
            }
            let q=crate::string(&s["profile"]["quorum"],"invalid-state")?.parse::<usize>().map_err(|_|err("invalid-state"))?;
            // A sole quorum already represented in verified history needs no
            // reconstruction. Competing historical quorums still halt below.
            let eligible:Vec<_>=groups.iter().filter(|(_,(_,v))|v.len()>=q).collect();
            if eligible.len()==1 && array(&s["history"],"invalid-state")?.iter().any(|h| h["finalityId"]==*eligible[0].0){continue;}
            if eligible.is_empty(){continue;}
            let basis=c::basis(&s,&head["finalityId"])?;
            let mut certs=vec![];
            for (p,votes) in groups.values().filter(|(_,v)|v.len()>=q){certs.push(c::assemble_certificate(&basis,p,votes)?);}
            if certs.len()>1 {
                self.append(j,&s,&json!({"kind":"coordination-conflict","certificates":&certs[..2]}))?;
                return Ok(json!({"status":"halted","reason":"conflicting-certificates"}));
            }
            if let Some(cert)=certs.first(){
                if array(&s["history"],"invalid-state")?.iter().any(|h|h["finalityId"]==cert["finalityId"]){continue;}
                let after=self.append(j,&s,&json!({"kind":"coordination-finality","certificate":cert}))?;
                return Ok(json!({"status":if after["halted"]==true{"halted"}else{"finalized"},"certificate":cert}));
            }
        }Ok(json!({"status":"unresolved","reason":"insufficient-votes"}))
    })
    }
}
