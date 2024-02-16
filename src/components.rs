use bevy::prelude::Component;
use common::tools::UnitsPerSecond;

#[derive(Component, Clone)]
pub enum VoidSpherePart {
	Core,
	RingA(UnitsPerSecond),
	RingB(UnitsPerSecond),
}

#[derive(Component)]
pub struct Mark<T>(pub T);

#[derive(Component)]
pub struct Dummy;
