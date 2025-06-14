use crate::traits::handles_custom_assets::TryLoadFrom;
use bevy::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

pub trait HandlesSaving {
	type TSaveEntityMarker: Component + Default;

	fn register_savable_component<TComponent>(app: &mut App)
	where
		TComponent: SavableComponent;
}

/// Marks components as being able to be (de)serialized.
///
/// A blanket implementation exists for components that can use `Self`
/// as [`TDto`](SavableComponent::TDto).
pub trait SavableComponent: Component + Sized + Clone + TryLoadFrom<Self::TDto> {
	/// The data transfer object used for (de)serialization.
	type TDto: From<Self> + Serialize + DeserializeOwned;
}

impl<T> SavableComponent for T
where
	T: Serialize + DeserializeOwned + Clone + Component,
{
	type TDto = Self;
}
