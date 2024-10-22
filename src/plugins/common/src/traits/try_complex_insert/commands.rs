use super::TryComplexInsert;
use bevy::prelude::*;

impl<'w, 's, TAsset> TryComplexInsert<Option<Handle<TAsset>>> for Commands<'w, 's>
where
	TAsset: Asset,
{
	fn try_complex_insert(&mut self, entity: Entity, handle: Option<Handle<TAsset>>) {
		let Some(mut entity) = self.get_entity(entity) else {
			return;
		};

		match handle {
			Some(handle) => {
				entity.try_insert(handle);
			}
			None => {
				entity.try_insert(Handle::<TAsset>::default());
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::new_handle;
	use bevy::ecs::system::RunSystemOnce;

	#[derive(Debug, PartialEq, Asset, TypePath)]
	struct _Asset;

	fn setup() -> App {
		App::new()
	}

	#[test]
	fn insert_handle() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let handle = new_handle::<_Asset>();
		let handle_cloned = handle.clone();

		app.world_mut()
			.run_system_once(move |mut commands: Commands| {
				commands.try_complex_insert(entity, Some(handle_cloned.clone()));
			});

		assert_eq!(
			Some(&handle),
			app.world().entity(entity).get::<Handle<_Asset>>()
		);
	}

	#[test]
	fn insert_default_handle_when_inserting_none() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut commands: Commands| {
				commands.try_complex_insert(entity, None as Option<Handle<_Asset>>);
			});

		assert_eq!(
			Some(&Handle::<_Asset>::default()),
			app.world().entity(entity).get::<Handle<_Asset>>()
		);
	}

	#[test]
	fn no_panic_when_entity_does_not_exist() {
		let mut app = setup();

		app.world_mut()
			.run_system_once(move |mut commands: Commands| {
				commands.try_complex_insert(Entity::from_raw(100), Some(new_handle::<_Asset>()));
			});
	}
}
