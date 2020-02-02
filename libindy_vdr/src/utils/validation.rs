use crate::common::error::VdrResult;

pub trait Validatable {
    fn validate(&self) -> VdrResult<()> {
        Ok(())
    }
}
