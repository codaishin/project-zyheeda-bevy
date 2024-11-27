use bevy::prelude::*;

pub trait HandlesDestruction {
	type TDestroy: Component + Default;
}
