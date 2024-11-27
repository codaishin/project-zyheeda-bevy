use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub enum Destroy {
	Immediately,
	AfterFrames(u8),
}

impl Destroy {
	pub const DELAYED: Destroy = Destroy::AfterFrames(2);
}
