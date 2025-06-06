pub mod force;
pub mod gravity;

use std::marker::PhantomData;

pub struct AffectedBy<TEffect>(PhantomData<TEffect>);

pub struct Affected;

impl Affected {
	pub fn by<TEffect>() -> AffectedBy<TEffect> {
		AffectedBy(PhantomData)
	}
}
