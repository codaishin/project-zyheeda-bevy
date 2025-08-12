use bevy::{
	ecs::{entity::EntityDoesNotExistError, system::SystemParam},
	prelude::*,
};

pub struct ZyheedaCommands<'w, 's> {
	commands: Commands<'w, 's>,
}

impl<'w, 's> ZyheedaCommands<'w, 's> {
	pub fn spawn<TBundle>(&mut self, bundle: TBundle) -> EntityCommands<'_>
	where
		TBundle: Bundle,
	{
		self.commands.spawn(bundle)
	}

	pub fn try_apply_on<TFn>(&mut self, entity: Entity, apply: TFn)
	where
		TFn: FnOnce(ZyheedaEntityCommands),
	{
		let Ok(entity) = self.get_entity(entity) else {
			return;
		};
		apply(entity)
	}

	pub fn insert_resource<TResource>(&mut self, resource: TResource)
	where
		TResource: Resource,
	{
		self.commands.insert_resource(resource);
	}

	pub fn get_entity(
		&mut self,
		entity: Entity,
	) -> Result<ZyheedaEntityCommands<'_>, EntityDoesNotExistError> {
		let entity = self.commands.get_entity(entity)?;
		Ok(ZyheedaEntityCommands { entity })
	}
}

unsafe impl<'w, 's> SystemParam for ZyheedaCommands<'w, 's> {
	type State = <Commands<'w, 's> as SystemParam>::State;
	type Item<'world, 'state> = ZyheedaCommands<'world, 'state>;

	fn init_state(
		world: &mut World,
		system_meta: &mut bevy::ecs::system::SystemMeta,
	) -> Self::State {
		Commands::<'w, 's>::init_state(world, system_meta)
	}

	unsafe fn get_param<'world, 'state>(
		state: &'state mut Self::State,
		system_meta: &bevy::ecs::system::SystemMeta,
		world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell<'world>,
		change_tick: bevy::ecs::component::Tick,
	) -> Self::Item<'world, 'state> {
		ZyheedaCommands {
			commands: unsafe {
				Commands::<'w, 's>::get_param(state, system_meta, world, change_tick)
			},
		}
	}

	unsafe fn new_archetype(
		state: &mut Self::State,
		archetype: &bevy::ecs::archetype::Archetype,
		system_meta: &mut bevy::ecs::system::SystemMeta,
	) {
		unsafe { Commands::<'w, 's>::new_archetype(state, archetype, system_meta) };
	}

	fn apply(
		state: &mut Self::State,
		system_meta: &bevy::ecs::system::SystemMeta,
		world: &mut World,
	) {
		Commands::<'w, 's>::apply(state, system_meta, world);
	}

	fn queue(
		state: &mut Self::State,
		system_meta: &bevy::ecs::system::SystemMeta,
		world: bevy::ecs::world::DeferredWorld,
	) {
		<Commands<'w, 's> as SystemParam>::queue(state, system_meta, world);
	}

	unsafe fn validate_param(
		state: &Self::State,
		system_meta: &bevy::ecs::system::SystemMeta,
		world: bevy::ecs::world::unsafe_world_cell::UnsafeWorldCell,
	) -> std::result::Result<(), bevy::ecs::system::SystemParamValidationError> {
		unsafe { Commands::<'w, 's>::validate_param(state, system_meta, world) }
	}
}

pub struct ZyheedaEntityCommands<'a> {
	entity: EntityCommands<'a>,
}

impl ZyheedaEntityCommands<'_> {
	pub fn try_insert<TBundle>(&mut self, bundle: TBundle) -> &mut Self
	where
		TBundle: Bundle,
	{
		self.entity.try_insert(bundle);
		self
	}

	pub fn try_insert_if_new<TBundle>(&mut self, bundle: TBundle) -> &mut Self
	where
		TBundle: Bundle,
	{
		self.entity.try_insert_if_new(bundle);
		self
	}

	pub fn try_remove<TBundle>(&mut self) -> &mut Self
	where
		TBundle: Bundle,
	{
		self.entity.try_remove::<TBundle>();
		self
	}

	pub fn try_despawn(mut self) {
		self.entity.try_despawn();
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::{SingleThreadedApp, assert_count};

	#[derive(Component, Debug, PartialEq)]
	struct _Component(&'static str);

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn spawn() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(|mut commands: ZyheedaCommands| {
				commands.spawn(());
			})?;

		assert_count!(1, app.world().iter_entities());
		Ok(())
	}

	#[test]
	fn spawn_with_bundle() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut()
			.run_system_once(|mut commands: ZyheedaCommands| {
				commands.spawn(_Component(""));
			})?;

		assert_count!(
			1,
			app.world()
				.iter_entities()
				.filter(|e| e.contains::<_Component>())
		);
		Ok(())
	}

	mod via_entity {
		use super::*;

		#[test]
		fn insert() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			app.world_mut()
				.run_system_once(move |mut commands: ZyheedaCommands| {
					let Ok(mut entity_cmds) = commands.get_entity(entity) else {
						return;
					};
					entity_cmds.try_insert(_Component(""));
				})?;

			assert_eq!(
				Some(&_Component("")),
				app.world().entity(entity).get::<_Component>()
			);
			Ok(())
		}

		#[test]
		fn no_error_on_insert_after_despawn() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			app.world_mut()
				.run_system_once(move |mut commands: ZyheedaCommands| {
					let Ok(mut entity_cmds) = commands.get_entity(entity) else {
						return;
					};

					entity_cmds.entity.despawn();
					entity_cmds.try_insert(_Component(""));
				})
		}

		#[test]
		fn insert_if_new() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			app.world_mut()
				.run_system_once(move |mut commands: ZyheedaCommands| {
					let Ok(mut entity_cmds) = commands.get_entity(entity) else {
						return;
					};
					entity_cmds.try_insert_if_new(_Component("new"));
				})?;

			assert_eq!(
				Some(&_Component("new")),
				app.world().entity(entity).get::<_Component>()
			);
			Ok(())
		}

		#[test]
		fn insert_if_new_no_override() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(_Component("old")).id();

			app.world_mut()
				.run_system_once(move |mut commands: ZyheedaCommands| {
					let Ok(mut entity_cmds) = commands.get_entity(entity) else {
						return;
					};
					entity_cmds.try_insert_if_new(_Component("new"));
				})?;

			assert_eq!(
				Some(&_Component("old")),
				app.world().entity(entity).get::<_Component>()
			);
			Ok(())
		}

		#[test]
		fn no_error_on_insert_if_new_after_despawn() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			app.world_mut()
				.run_system_once(move |mut commands: ZyheedaCommands| {
					let Ok(mut entity_cmds) = commands.get_entity(entity) else {
						return;
					};

					entity_cmds.entity.despawn();
					entity_cmds.try_insert_if_new(_Component("new"));
				})
		}

		#[test]
		fn remove() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(_Component("")).id();

			app.world_mut()
				.run_system_once(move |mut commands: ZyheedaCommands| {
					let Ok(mut entity_cmds) = commands.get_entity(entity) else {
						return;
					};
					entity_cmds.try_remove::<_Component>();
				})?;

			assert_eq!(None, app.world().entity(entity).get::<_Component>());
			Ok(())
		}

		#[test]
		fn despawn() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			app.world_mut()
				.run_system_once(move |mut commands: ZyheedaCommands| {
					let Ok(entity_cmds) = commands.get_entity(entity) else {
						return;
					};
					entity_cmds.try_despawn();
				})?;

			assert!(app.world().get_entity(entity).is_err());
			Ok(())
		}
	}

	mod via_commands {
		use super::*;

		#[test]
		fn try_apply() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			app.world_mut()
				.run_system_once(move |mut commands: ZyheedaCommands| {
					commands.try_apply_on(entity, |mut e| {
						e.try_insert(_Component(""));
					});
				})?;

			assert_eq!(
				Some(&_Component("")),
				app.world().entity(entity).get::<_Component>()
			);
			Ok(())
		}

		#[test]
		fn no_error_on_try_apply_after_despawn() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn_empty().id();

			app.world_mut()
				.run_system_once(move |mut commands: ZyheedaCommands| {
					let Ok(mut entity_cmds) = commands.get_entity(entity) else {
						return;
					};

					entity_cmds.entity.despawn();
					commands.try_apply_on(entity, |mut e| {
						e.try_insert(_Component(""));
					});
				})
		}

		#[test]
		fn insert_resource() -> Result<(), RunSystemError> {
			#[derive(Resource, Debug, PartialEq)]
			struct _Resource;

			let mut app = setup();

			app.world_mut()
				.run_system_once(|mut commands: ZyheedaCommands| {
					commands.insert_resource(_Resource);
				})?;

			assert_eq!(Some(&_Resource), app.world().get_resource::<_Resource>());
			Ok(())
		}
	}
}
