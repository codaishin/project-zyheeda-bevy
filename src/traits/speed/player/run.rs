use crate::{
	behavior::Run,
	components::{Player, UnitsPerSecond},
	traits::speed::Speed,
};

impl Speed<Run> for Player {
	fn get_speed(&self) -> UnitsPerSecond {
		self.run_speed
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;

	fn get_player(speed: f32) -> impl Speed<Run> {
		Player {
			run_speed: UnitsPerSecond::new(speed),
			..default()
		}
	}

	#[test]
	fn get_run_speed() {
		let player = get_player(42.);

		assert_eq!(UnitsPerSecond::new(42.), player.get_speed());
	}
}
