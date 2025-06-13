mod external_components;

use crate::traits::handles_custom_assets::TryLoadFrom;
use bevy::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

pub trait HandlesSaving {
	type TSaveEntityMarker: Component + Default;

	fn register_savable_component<TComponent>(app: &mut App)
	where
		TComponent: SavableComponent;
}

pub trait SavableComponent: Component + Sized + Clone + TryLoadFrom<Self::TDto> {
	/// The data transfer object used for (de)serialization.
	///
	/// In the simplest case this can be `Self`.
	type TDto: From<Self> + Serialize + DeserializeOwned;
}
