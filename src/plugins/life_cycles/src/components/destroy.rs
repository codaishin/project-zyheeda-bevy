use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
pub enum Destroy {
	#[default]
	Immediately,
	AfterFrames(u8),
}

impl Destroy {
	pub const DELAYED: Destroy = Destroy::AfterFrames(2);
}
