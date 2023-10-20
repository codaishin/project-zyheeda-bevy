use crate::{
	behavior::Run,
	components::{Player, UnitsPerSecond},
	traits::speed::Speed,
};

impl Speed<Run> for Player {
	fn get_speed(&self) -> UnitsPerSecond {
		todo!()
	}
}
