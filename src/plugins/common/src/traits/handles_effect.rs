use bevy::prelude::Bundle;

pub trait HandlesEffect<TEffect> {
	fn effect(effect: TEffect) -> impl Bundle;
}
