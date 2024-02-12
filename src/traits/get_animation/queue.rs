use super::HasIdle;
use crate::components::Animate;
use common::{
	components::{Queue, Side, SideUnset},
	skill::PlayerSkills,
};

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
