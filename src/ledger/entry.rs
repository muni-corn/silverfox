use crate::ledger::{Amount};

pub struct Entry {
    date: chrono::Date<chrono::Local>,
    description: String,
    postings: Vec<Posting>,
    envelope_postings: Vec<EnvelopePosting>
}

pub struct Posting {
    amount: Amount,
    account: String,
    price_assertion: Option<Amount>,
    posting_type: PostingType,
    balance_assertion: Option<Amount>
}

pub struct EnvelopePosting {
    amount: Amount,
    account: String
}

pub enum PostingType {
    Real,
    BalancedVirtual,
    UnbalancedVirtual
}
