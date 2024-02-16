use super::HasIdle;
use crate::{
	components::{Queue, SideUnset},
	skill::PlayerSkills,
};
use common::components::{Animate, Side};

impl HasIdle<PlayerSkills<Side>> for Queue<PlayerSkills<SideUnset>> {
	const IDLE: Animate<PlayerSkills<Side>> = Animate::Repeat(PlayerSkills::Idle);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_idle() {
		assert_eq!(
			Animate::<PlayerSkills<Side>>::Repeat(PlayerSkills::Idle),
			Queue::IDLE
		);
	}
}
