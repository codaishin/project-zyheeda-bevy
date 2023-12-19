pub mod tools;

use crate::components::{Active, Queued, Skill};

pub trait TrySoftOverride {
	fn try_soft_override(
		running: &Skill<Active>,
		new: &Skill<Queued>,
	) -> Option<(Skill<Active>, Skill<Queued>)>;
}
