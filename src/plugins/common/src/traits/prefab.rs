mod app;
mod entity_commands;

use crate::{errors::ErrorData, traits::load_asset::LoadAsset};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

pub trait Prefab<TDependency>: Component {
	type TError: ErrorData;

	const REAPPLY: Reapply = Reapply::Never;

	fn insert_prefab_components(
		&self,
		entity: &mut impl PrefabEntityCommands,
		assets: &mut impl LoadAsset,
	) -> Result<(), Self::TError>;
}

/// Prefab application strategy:
///
/// - [`Reapply::Never`]: [`Prefab`] is not re-applied when it is re-inserted
/// - [`Reapply::Always`]: [`Prefab`] is re-applied each time it is inserted
///
/// The default for [`Prefab`] is [`Reapply::Never`].
///
/// <div class="warning">
///   Using `Reapply::Always` is dangerous when the prefab may insert different component types
///   and/or adds children. In this case a scheme to cleanup outdated components/children is
///   required.
/// </div>
#[derive(Debug, PartialEq)]
pub enum Reapply {
	Never,
	Always,
}

pub trait AddPrefabObserver {
	fn add_prefab_observer<TPrefab, TDependencies>(&mut self) -> &mut Self
	where
		TPrefab: Prefab<TDependencies> + Component,
		TDependencies: 'static;
}

pub trait TryInsert {
	fn try_insert<TBundle>(&mut self, bundle: TBundle) -> &mut Self
	where
		TBundle: Bundle;
}

pub trait TryInsertIfNew {
	fn try_insert_if_new<TBundle>(&mut self, bundle: TBundle) -> &mut Self
	where
		TBundle: Bundle;
}

pub trait TryRemove {
	fn try_remove<TBundle>(&mut self) -> &mut Self
	where
		TBundle: Bundle;
}

pub trait WithChild {
	fn with_child<TBundle>(&mut self, bundle: TBundle) -> &mut Self
	where
		TBundle: Bundle;
}

pub trait WithChildren {
	fn with_children<TFunc>(&mut self, func: TFunc) -> &mut Self
	where
		TFunc: FnOnce(&mut RelatedSpawnerCommands<ChildOf>);
}

pub trait PrefabEntityCommands:
	TryInsert + TryInsertIfNew + TryRemove + WithChild + WithChildren
{
}

impl<T> PrefabEntityCommands for T where
	T: TryInsert + TryInsertIfNew + TryRemove + WithChild + WithChildren
{
}
