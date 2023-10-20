use crate::{
	behavior::Walk,
	components::{Player, UnitsPerSecond},
	traits::speed::Speed,
};

impl Speed<Walk> for Player {
	fn get_speed(&self) -> UnitsPerSecond {
		self.walk_speed
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;

	fn get_player(speed: f32) -> impl Speed<Walk> {
		Player {
			walk_speed: UnitsPerSecond::new(speed),
			..default()
		}
	}

	#[test]
	fn get_walk_speed() {
		let player = get_player(42.);

		assert_eq!(UnitsPerSecond::new(42.), player.get_speed());
	}
}
