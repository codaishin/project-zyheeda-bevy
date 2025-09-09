use bevy::prelude::*;

macro_rules! impl_savable_with_high_priority {
	($ty:ty) => {
		impl $crate::traits::handles_saving::SavableComponent for $ty {
			type TDto = $ty;
			const PRIORITY: bool = true;
		}
	};
	($ty:ty, $($rest:ty),+ $(,)?) => {
		impl_savable_with_high_priority!($ty);
		impl_savable_with_high_priority!($($rest),+);
	};
}

impl_savable_with_high_priority!(Transform, Name);
