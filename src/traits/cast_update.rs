pub mod skill;

use std::time::Duration;

#[derive(PartialEq, Debug)]
pub enum CastType {
	Pre,
	After,
}

#[derive(PartialEq, Debug)]
pub enum State {
	New,
	Activate,
	Done,
	Casting(CastType),
}
pub trait CastUpdate {
	fn update(&mut self, delta: Duration) -> State;
}
