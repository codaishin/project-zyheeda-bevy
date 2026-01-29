mod external;

use crate::{errors::Unreachable, traits::handles_custom_assets::TryLoadFrom};
use bevy::prelude::*;
use serde::{Serialize, de::DeserializeOwned};

pub trait HandlesSaving {
	type TSaveEntityMarker: Component + Default;

	/// Check whether quick loading is possible
	///
	/// Useful for button (dis|en)ables.
	fn can_quick_load() -> impl SystemCondition<()>;

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

	/// Whether this component should be loaded before non priority components from a save file
	const PRIORITY: bool = false;
}

#[cfg(test)]
mod test_savable_component_derive {
	use super::*;
	use macros::SavableComponent;
	use serde::Deserialize;
	use std::any::TypeId;

	#[derive(Component, SavableComponent, Clone, Serialize, Deserialize)]
	struct _Default;

	#[derive(Component, SavableComponent, Clone)]
	#[savable_component(dto = _Dto)]
	struct _WithDto;

	#[derive(Serialize, Deserialize)]
	struct _Dto;

	impl From<_WithDto> for _Dto {
		fn from(_: _WithDto) -> Self {
			Self
		}
	}

	impl TryLoadFrom<_Dto> for _WithDto {
		type TInstantiationError = Unreachable;

		fn try_load_from<TLoadAsset>(
			_: _Dto,
			_: &mut TLoadAsset,
		) -> Result<Self, Self::TInstantiationError> {
			Ok(Self)
		}
	}

	#[derive(Component, SavableComponent, Clone, Serialize, Deserialize)]
	#[savable_component(has_priority)]
	struct _Priority;

	#[test]
	fn default_to_self_as_dto() {
		assert_eq!(
			TypeId::of::<_Default>(),
			TypeId::of::<<_Default as SavableComponent>::TDto>()
		);
	}

	#[test]
	#[allow(clippy::assertions_on_constants)]
	fn default_to_no_priority() {
		assert!(!_Default::PRIORITY);
	}

	#[test]
	fn has_dto() {
		assert_eq!(
			TypeId::of::<_Dto>(),
			TypeId::of::<<_WithDto as SavableComponent>::TDto>()
		);
	}

	#[test]
	#[allow(clippy::assertions_on_constants)]
	fn has_priority() {
		assert!(_Priority::PRIORITY);
	}
}
