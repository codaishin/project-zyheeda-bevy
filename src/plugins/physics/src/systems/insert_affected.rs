use std::{any::type_name, marker::PhantomData};

use bevy::prelude::*;
use common::{
	errors::{Error, Level},
	tools::attribute::AttributeOnSpawn,
	traits::{
		accessors::get::{RefAs, RefInto, TryApplyOn},
		handles_agents::RefIntoAssetHandle,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> InsertAffected for T where T: AffectedComponent {}

pub(crate) trait InsertAffected: AffectedComponent {
	fn insert_on<TSource>(
		mut commands: ZyheedaCommands,
		sources: Query<(Entity, &TSource), Without<Self>>,
		assets: Res<Assets<TSource::TAsset>>,
	) -> Result<(), Vec<AttributeAssetNotFound<TSource::TAsset>>>
	where
		TSource: Component + RefIntoAssetHandle,
		TSource::TAsset: for<'a> RefInto<'a, AttributeOnSpawn<Self::TAttribute>>,
	{
		let mut errors = vec![];

		for (entity, source) in &sources {
			let Ok(handle) = source.ref_into_asset_handle() else {
				continue;
			};
			let Some(asset) = assets.get(handle) else {
				errors.push(AttributeAssetNotFound::from(entity));
				continue;
			};

			commands.try_apply_on(&entity, |mut e| {
				let attribute = asset.ref_as::<AttributeOnSpawn<Self::TAttribute>>();
				e.try_insert(Self::from(attribute));
			});
		}

		if !errors.is_empty() {
			return Err(errors);
		}

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
pub(crate) struct AttributeAssetNotFound<TAsset> {
	entity: Entity,
	_p: PhantomData<TAsset>,
}

impl<TAsset> From<Entity> for AttributeAssetNotFound<TAsset> {
	fn from(entity: Entity) -> Self {
		Self {
			entity,
			_p: PhantomData,
		}
	}
}

impl<TAsset> From<AttributeAssetNotFound<TAsset>> for Error {
	fn from(AttributeAssetNotFound { entity, .. }: AttributeAssetNotFound<TAsset>) -> Self {
		let typename = type_name::<TAsset>();
		Error::Single {
			msg: format!("{entity}: attribute asset {typename} not found"),
			lvl: Level::Error,
		}
	}
}

pub(crate) trait AffectedComponent:
	Component + From<AttributeOnSpawn<Self::TAttribute>>
{
	type TAttribute;
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::handles_agents::AgentAssetNotLoaded;
	use testing::{SingleThreadedApp, new_handle};

	#[derive(Component, Debug, PartialEq)]
	struct _Affected(_Attribute);

	impl From<AttributeOnSpawn<_Attribute>> for _Affected {
		fn from(AttributeOnSpawn(attribute): AttributeOnSpawn<_Attribute>) -> Self {
			Self(attribute)
		}
	}

	impl AffectedComponent for _Affected {
		type TAttribute = _Attribute;
	}

	#[derive(Component, Debug, PartialEq, Clone, Copy)]
	struct _Attribute(&'static str);

	#[derive(Component)]
	struct _Source(Option<Handle<_Asset>>);

	impl RefIntoAssetHandle for _Source {
		type TAsset = _Asset;

		fn ref_into_asset_handle(&self) -> Result<&'_ Handle<Self::TAsset>, AgentAssetNotLoaded> {
			match &self.0 {
				Some(handle) => Ok(handle),
				None => Err(AgentAssetNotLoaded),
			}
		}
	}

	#[derive(Asset, TypePath, Debug, PartialEq)]
	struct _Asset(_Attribute);

	impl<'a> RefInto<'a, AttributeOnSpawn<_Attribute>> for _Asset {
		fn ref_into(&'a self) -> AttributeOnSpawn<_Attribute> {
			AttributeOnSpawn(self.0)
		}
	}

	fn setup<const N: usize>(assets: [(&Handle<_Asset>, _Asset); N]) -> App {
		let mut app = App::new().single_threaded(Update);
		let mut loaded_assets = Assets::default();

		for (id, asset) in assets {
			loaded_assets.insert(id, asset);
		}

		app.insert_resource(loaded_assets);

		app
	}

	#[test]
	fn insert_when_asset_loaded() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let mut app = setup([(&handle, _Asset(_Attribute("my attribute")))]);
		let entity = app.world_mut().spawn(_Source(Some(handle))).id();

		_ = app
			.world_mut()
			.run_system_once(_Affected::insert_on::<_Source>)?;

		assert_eq!(
			Some(&_Affected(_Attribute("my attribute"))),
			app.world().entity(entity).get::<_Affected>(),
		);
		Ok(())
	}

	#[test]
	fn do_not_insert_when_affected_already_present() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let mut app = setup([(&handle, _Asset(_Attribute("my new attribute")))]);
		let entity = app
			.world_mut()
			.spawn((
				_Source(Some(handle)),
				_Affected(_Attribute("already inserted attribute")),
			))
			.id();

		_ = app
			.world_mut()
			.run_system_once(_Affected::insert_on::<_Source>)?;

		assert_eq!(
			Some(&_Affected(_Attribute("already inserted attribute"))),
			app.world().entity(entity).get::<_Affected>(),
		);
		Ok(())
	}

	#[test]
	fn return_ok_when_asset_loaded() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let mut app = setup([(&handle, _Asset(_Attribute("my attribute")))]);
		app.world_mut().spawn(_Source(Some(handle)));

		let result = app
			.world_mut()
			.run_system_once(_Affected::insert_on::<_Source>)?;

		assert_eq!(Ok(()), result);
		Ok(())
	}

	#[test]
	fn return_orr_when_asset_not_found() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let mut app = setup([]);
		let entity = app.world_mut().spawn(_Source(Some(handle))).id();

		let result = app
			.world_mut()
			.run_system_once(_Affected::insert_on::<_Source>)?;

		assert_eq!(Err(vec![AttributeAssetNotFound::from(entity)]), result);
		Ok(())
	}
}
