use std::collections::HashMap;
use crate::ledger::Amount;
use chrono::{Date, Local};

pub struct Account {
    name: &'static str,
    envelopes: HashMap<&'static str, Envelope>
}

pub struct Envelope {
    name: String,
    amount: Amount,
    envelope_type: EnvelopeType,
    freq: Frequency,
    next_due_date: Date<Local>,
    funding: Funding
}

pub enum EnvelopeType {
    Expense,
    Goal,
}

pub enum Funding {
    Conservative,
    Aggressive,
}

pub enum Frequency {
    Never,
    Once,
    Weekly,
    Biweekly,
    Monthly,
    Bimonthly,
    Quarterly,
    Semiannually,
    Annually
}

