use crate::{
	behavior::Walk,
	components::{Player, UnitsPerSecond},
	traits::speed::Speed,
};

impl Speed<Walk> for Player {
	fn get_speed(&self) -> UnitsPerSecond {
		todo!()
	}
}
