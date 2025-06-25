use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

macro_rules! impl_savable {
	($ty:ty) => {
		impl $crate::traits::handles_saving::SavableComponent for $ty {
			type TDto = $ty;
			const PRIORITY: bool = false;
		}
	};
	($ty:ty, $($rest:ty),+ $(,)?) => {
		impl_savable!($ty);
		impl_savable!($($rest),+);
	};
}

impl_savable!(Transform, Name, Velocity);
