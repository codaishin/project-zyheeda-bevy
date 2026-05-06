mod external;

use crate::{errors::Unreachable, traits::handles_custom_assets::TryLoadFrom};
use bevy::prelude::*;
use serde::{Serialize, de::DeserializeOwned};
use std::{hash::Hash, ops::Deref, sync::OnceLock};

pub trait HandlesSaving {
	type TSaveEntityMarker: Component + Default;

	/// Check whether quick loading is possible
	///
	/// Useful for button (dis|en)ables.
	fn can_quick_load() -> impl SystemCondition<()>;

	/// Register savable components.
	///
	/// Implementors are likely to panic if uniqueness of [`SavableComponent::ID`] is violated.
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

	/// Identifier for component saving/loading
	const ID: UniqueComponentId;
}

/// A unique id for a component.
///
/// Uniqueness:
/// - must be enforced by users of `SavableComponent`.
/// - is likely essential. Meaning that uniqueness violations may crash the application, thus,
///   checking on app startup is desired.
#[derive(Debug, Eq, Clone)]
pub enum UniqueComponentId {
	Id(&'static str),
	IdLazy {
		id: OnceLock<String>,
		f: fn() -> String,
	},
}

use UniqueComponentId::{Id, IdLazy};

impl UniqueComponentId {
	pub const fn from_str(id: &'static str) -> Self {
		Id(id)
	}

	pub const fn from_lazy(f: fn() -> String) -> Self {
		IdLazy {
			id: OnceLock::new(),
			f,
		}
	}
}

impl PartialEq for UniqueComponentId {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Id(l), Id(r)) => l == r,
			(Id(id_str), IdLazy { id, f }) | (IdLazy { id, f }, Id(id_str)) => {
				id_str == id.get_or_init(f)
			}
			(IdLazy { id: id_l, f: f_l }, IdLazy { id: id_r, f: f_r }) => {
				id_l.get_or_init(f_l) == id_r.get_or_init(f_r)
			}
		}
	}
}

impl Hash for UniqueComponentId {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			Id(id) => id.hash(state),
			IdLazy { id, f } => id.get_or_init(f).hash(state),
		}
	}
}

impl Deref for UniqueComponentId {
	type Target = str;

	fn deref(&self) -> &Self::Target {
		match self {
			Id(id) => id,
			IdLazy { id, f } => id.get_or_init(f),
		}
	}
}

impl From<&'static str> for UniqueComponentId {
	fn from(id: &'static str) -> Self {
		Self::from_str(id)
	}
}

impl From<&UniqueComponentId> for String {
	fn from(id: &UniqueComponentId) -> Self {
		match id {
			Id(r) => (*r).to_owned(),
			IdLazy { id, f } => id.get_or_init(f).clone(),
		}
	}
}

impl From<UniqueComponentId> for String {
	fn from(id: UniqueComponentId) -> Self {
		Self::from(&id)
	}
}

#[cfg(test)]
mod test_savable_component_derive {
	use super::*;
	use macros::SavableComponent;
	use serde::Deserialize;
	use std::any::TypeId;

	#[derive(Component, SavableComponent, Clone, Serialize, Deserialize)]
	#[savable_component(id = "default")]
	struct _Default;

	#[derive(Component, SavableComponent, Clone)]
	#[savable_component(id = "with dto", dto = _Dto)]
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
	#[savable_component(id = "priority", has_priority)]
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
