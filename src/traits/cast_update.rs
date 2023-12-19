pub mod track;

use std::time::Duration;

#[derive(PartialEq, Debug, Clone)]
pub enum CastType {
	Pre,
	After,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AgeType {
	New,
	Old,
}

#[derive(PartialEq, Debug, Clone)]
pub enum State {
	New,
	Active(AgeType),
	Done,
	Casting(CastType),
}
pub trait CastUpdate {
	fn update(&mut self, delta: Duration) -> State;
}
