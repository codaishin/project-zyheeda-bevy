use bevy::prelude::*;
use common::{
	tools::attribute::AttributeOnSpawn,
	traits::accessors::get::{
		AssociatedItem,
		AssociatedStaticSystemParam,
		GetFromSystemParam,
		RefAs,
		RefInto,
		TryApplyOn,
	},
	zyheeda_commands::ZyheedaCommands,
};

impl<T> InsertAffected for T where T: AffectedComponent {}

pub(crate) trait InsertAffected: AffectedComponent {
	fn insert_on<TSource>(
		mut commands: ZyheedaCommands,
		sources: Query<(Entity, &TSource), Without<Self>>,
		param: AssociatedStaticSystemParam<TSource, ()>,
	) where
		for<'w, 's> TSource: Component + GetFromSystemParam<'w, 's, ()>,
		for<'w, 's, 'i> AssociatedItem<'w, 's, 'i, TSource, ()>:
			RefInto<'i, AttributeOnSpawn<Self::TAttribute>>,
	{
		for (entity, source) in &sources {
			let param = &param;
			let Some(config) = source.get_from_param(&(), param) else {
				continue;
			};

			_ = config.ref_into();
			// let s = Self::from(attribute);

			// commands.try_apply_on(&entity, move |mut e| {
			// 	e.try_insert(s);
			// });
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
		app.add_systems(Update, _Affected::insert_on::<_Source>);

		app
	}

	#[test]
	fn insert_when_asset_loaded() -> Result<(), RunSystemError> {
		let handle = new_handle();
		let mut app = setup([(&handle, _Asset(_Attribute("my attribute")))]);
		let entity = app.world_mut().spawn(_Source(Some(handle))).id();

		app.update();

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

		app.update();

		assert_eq!(
			Some(&_Affected(_Attribute("already inserted attribute"))),
			app.world().entity(entity).get::<_Affected>(),
		);
		Ok(())
	}
}
