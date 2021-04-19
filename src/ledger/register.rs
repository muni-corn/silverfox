use crate::{amount::AmountPool, entry::Entry, entry::EntryRegisterData, errors::SilverfoxError};
use chrono::NaiveDate;

pub struct Register;

impl Register {
    pub fn display(
        entries: &[Entry],
        date_format: &str,
        begin_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
        account_match: Option<String>,
    ) -> Result<(), SilverfoxError> {
        let console_width = if let Some(s) = terminal_size::terminal_size() {
            (s.0).0 as usize
        } else {
            return Err(SilverfoxError::Basic(String::from(
                "couldn't figure out the width of your terminal. are you in a terminal?",
            )));
        };

        // a "focused" account is the focus of the register. in other words, numbers displayed
        // revolve around the focused account. if money flows into the account, it is displayed as
        // a positive number on the register. if money flows out, it is displayed as a negative
        // number.
        let is_account_name_focused = |account_name: &str| match &account_match {
            Some(match_str) => account_name.contains(match_str),
            // TODO: an issue ticket is open to further solidify whether or not an account is an
            // "asset", so this will be changed soon (it's kinda dumb right now)
            None => account_name.starts_with("asset"),
        };

        let filtered: Vec<&Entry> = entries
            .iter()
            .filter(|e| {
                let has_focused_account = e
                    .get_postings()
                    .iter()
                    .any(|p| is_account_name_focused(p.get_account()));

                let date_in_range = match begin_date {
                    Some(begin) => match end_date {
                        Some(end) => e.get_date() <= &end && e.get_date() >= &begin,
                        None => e.get_date() >= &begin,
                    },
                    None => match end_date {
                        Some(end) => e.get_date() <= &end,
                        None => true,
                    },
                };

                // entries must have at least one focused account and be within the range between the
                // start date and end date (both inclusive)
                has_focused_account && date_in_range
            })
            .collect();

        let mut register_data_vec = Vec::new();

        let maximums = get_maximum_lengths(
            &filtered,
            date_format,
            account_match,
            &mut register_data_vec,
        )?;

        print_lines(&maximums, &register_data_vec, console_width);

        Ok(())
    }
}

#[derive(Default)]
struct MaximumLens {
    date: usize,
    description: usize,
    long_from_account: usize,
    long_to_account: usize,
    short_from_account: usize,
    short_to_account: usize,
    single_account: usize,
    amount: usize,
    running_total: usize,
}

fn get_maximum_lengths(
    filtered_entries: &[&Entry],
    date_format: &str,
    account_match: Option<String>,
    register_data_vec: &mut Vec<EntryRegisterData>,
) -> Result<MaximumLens, SilverfoxError> {
    let mut m = MaximumLens::default();

    let mut running_total = AmountPool::new();

    for entry in filtered_entries {
        let reg_data = match entry.as_register_data(date_format, &account_match) {
            Ok(o) => {
                if let Some(r) = o {
                    if !r.amounts.is_empty() {
                        r
                    } else {
                        continue;
                    }
                } else {
                    continue;
                }
            }
            Err(e) => {
                return Err(SilverfoxError::Basic(format!(
                    "couldn't display a register:\n\n{}",
                    e
                )))
            }
        };

        m.date = m.date.max(reg_data.date.len());
        m.description = m.description.max(reg_data.description.len());
        m.long_from_account = m.long_from_account.max(reg_data.account_flow.0.len());
        m.long_to_account = m.long_to_account.max(reg_data.account_flow.1.len());
        m.short_from_account = m
            .short_from_account
            .max(reg_data.short_account_flow.0.len());
        m.short_to_account = m.short_to_account.max(reg_data.short_account_flow.1.len());
        m.single_account = m.single_account.max(reg_data.single_account_display.len());
        m.amount = m.amount.max(
            reg_data
                .amounts
                .iter()
                .map(|a| format!("{}", a).len())
                .max()
                .unwrap(),
        );

        running_total += &reg_data.amounts;
        m.running_total = m.running_total.max(
            running_total
                .iter()
                .map(|a| format!("{}", a).len())
                .max()
                .unwrap(),
        );

        register_data_vec.push(reg_data);
    }

    Ok(m)
}

fn print_lines(maximums: &MaximumLens, register_data: &[EntryRegisterData], console_width: usize) {
    let mut running_total = AmountPool::new();

    for rd in register_data {
        running_total += &rd.amounts;

        let mut amount_iter = rd.amounts.iter();

        if let Some(first_amount) = amount_iter.next() {
            let prelude = format!(
                "{:date_len$} {} {:description_len$}  {:>from_acct_len$} -> {:to_acct_len$}  ",
                rd.date,
                rd.status,
                rd.description,
                rd.account_flow.0,
                rd.account_flow.1,
                date_len = maximums.date,
                description_len = maximums.description,
                from_acct_len = maximums.long_from_account,
                to_acct_len = maximums.long_to_account,
            );

            // TODO: Have Amount::display handle formatting arguments
            print!("{}", prelude);
            println!(
                "{:>amount_len$}  {:>running_total_len$}",
                format!("{}", first_amount),
                format!("{}", running_total.only(&first_amount.symbol)),
                amount_len = maximums.amount,
                running_total_len = maximums.running_total,
            );

            let prelude_space = spaces(prelude.len());
            for amount in amount_iter {
                println!(
                    "{}{:>amount_len$}  {:>running_total_len$}",
                    prelude_space,
                    format!("{}", amount),
                    format!("{}", running_total.only(&amount.symbol)),
                    amount_len = maximums.amount,
                    running_total_len = maximums.running_total,
                );
            }
        }
    }
}

fn spaces(n: usize) -> String {
    " ".repeat(n)
}
