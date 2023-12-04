pub mod tools;

use crate::components::{Active, Queued, Skill};

pub trait TryChain {
	fn try_chain(running: &mut Skill<Active>, new: &mut Skill<Queued>) -> bool;
}
