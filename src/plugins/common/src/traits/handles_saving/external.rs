use bevy::prelude::*;

macro_rules! impl_savable_with_high_priority {
	($ty:ty, $id:literal $(,)?) => {
		impl $crate::traits::handles_saving::SavableComponent for $ty {
			type TDto = $ty;
			const PRIORITY: bool = true;
			const ID: $crate::traits::handles_saving::UniqueComponentId =
				$crate::traits::handles_saving::UniqueComponentId($id);
		}
	};
}

impl_savable_with_high_priority!(Transform, "transform");
impl_savable_with_high_priority!(Name, "name");
