use crate::components::UnitsPerSecond;
pub mod player;

pub trait Speed {
	fn get_speed(&self) -> UnitsPerSecond;
}
