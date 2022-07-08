use crate::amount::AmountPool;
use crate::envelope::builder::EnvelopeBuilder;
use crate::errors::{SilverfoxError, SilverfoxResult};

use super::Account;

pub struct AccountBuilder {
    name: String,
    envelope_builders: Vec<EnvelopeBuilder>,
}

impl AccountBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            envelope_builders: Vec::new(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn envelope(mut self, builder: EnvelopeBuilder) -> Self {
        self.envelope_builders.push(builder);
        self
    }

    pub fn build(self) -> SilverfoxResult<Account> {
        let envelopes =
            self.envelope_builders
                .into_iter()
                .try_fold(Vec::new(), |mut acc, builder| {
                    // try to build the envelope and then push it (if successful)
                    acc.push(builder.build()?);
                    Ok::<_, SilverfoxError>(acc)
                })?;

        Ok(Account {
            name: self.name,
            envelopes,
            real_value: AmountPool::new(),
        })
    }
}
