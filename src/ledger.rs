use chrono;

pub enum CurrencyAlignment {
    Prefix,
    Postfix
}

pub struct Entry {
    date: chrono::Date<chrono::Local>,
    description: &'static str,
    postings: Vec<Posting>
}

pub struct Ledger {
    entries: Vec<Entry>
}

pub struct Posting {
    amount: Amount,
    account: &'static str,
    native_price: f64
}

pub struct Amount {
    mag: f32,
    currency: &'static str,
    currency_alignment: CurrencyAlignment
}

impl Entry {

}
