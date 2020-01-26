use crate::common::error::LedgerResult;

pub trait Validatable {
    fn validate(&self) -> LedgerResult<()> {
        Ok(())
    }
}
