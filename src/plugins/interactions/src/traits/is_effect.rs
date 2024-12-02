use bevy::prelude::*;

pub trait IsEffect {
	type TTarget;
	type TTargetComponent: Component;

	fn attribute(target_attribute: Self::TTarget) -> Self::TTargetComponent;
}
