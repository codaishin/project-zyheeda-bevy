mod external;

use crate::{errors::Unreachable, traits::handles_custom_assets::TryLoadFrom};
use bevy::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

pub trait HandlesSaving {
	type TSaveEntityMarker: Component + Default;

	fn register_savable_component<TComponent>(app: &mut App)
	where
		TComponent: SavableComponent;
}

/// Marks components as being (de)serializable.
///
/// A blanket implementation exists for components that can use `Self`
/// as their [`TDto`](SavableComponent::TDto).
///
/// <div class="warning">
///   `TInstantiationError` is hardcoded to [`Unreachable`] to simplify implementations.
///   While it could be constrained to types castable to `crate::errors::Error`,
///   doing so would introduce considerable boilerplate throughout the codebase,
///   including in awkward or fragile areas.
/// </div>
pub trait SavableComponent:
	Component + Sized + Clone + TryLoadFrom<Self::TDto, TInstantiationError = Unreachable>
{
	/// The data transfer object used for (de)serialization.
	type TDto: From<Self> + Serialize + DeserializeOwned;

	/// Weather this component should be loaded before non priority components from a save file
	const PRIORITY: bool = false;
}

/// Implements [`SavableComponent`] for the provided type(s) with:
/// - `TDto = Self`
/// - `PRIORITY = false`
#[macro_export]
macro_rules! impl_savable_self_non_priority {
	($ty:ty) => {
		impl $crate::traits::handles_saving::SavableComponent for $ty {
			type TDto = $ty;
			const PRIORITY: bool = false;
		}
	};
	($ty:ty, $($rest:ty),+ $(,)?) => {
		impl_savable_self_non_priority!($ty);
		impl_savable_self_non_priority!($($rest),+);
	};
}
