use crate::components::UnitsPerSecond;
pub mod player;

pub trait Speed<TMode> {
	fn get_speed(&self) -> UnitsPerSecond;
}
