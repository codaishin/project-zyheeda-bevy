pub mod tools;

use crate::components::{Active, Queued, Skill};

pub trait TrySoftOverride {
	fn try_soft_override(running: &mut Skill<Active>, new: &mut Skill<Queued>) -> bool;
}
