use crate::traits::prefab::{TryInsertIfNew, WithChild, WithChildren};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

impl TryInsertIfNew for EntityCommands<'_> {
	fn try_insert_if_new<TBundle>(&mut self, bundle: TBundle) -> &mut Self
	where
		TBundle: Bundle,
	{
		EntityCommands::try_insert_if_new(self, bundle)
	}
}

impl WithChild for EntityCommands<'_> {
	fn with_child<TBundle>(&mut self, bundle: TBundle) -> &mut Self
	where
		TBundle: Bundle,
	{
		EntityCommands::with_child(self, bundle)
	}
}

impl WithChildren for EntityCommands<'_> {
	fn with_children<TFunc>(&mut self, func: TFunc) -> &mut Self
	where
		TFunc: FnOnce(&mut RelatedSpawnerCommands<ChildOf>),
	{
		EntityCommands::with_children(self, func)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assert_count, get_children, test_tools::utils::SingleThreadedApp};
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Component(&'static str);

	fn try_insert_if_new_system(entity: Entity, component: _Component) -> impl Fn(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);

			<EntityCommands as TryInsertIfNew>::try_insert_if_new(&mut entity, component.clone());
		}
	}

	fn with_child_system(entity: Entity, component: _Component) -> impl Fn(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);

			<EntityCommands as WithChild>::with_child(&mut entity, component.clone());
		}
	}

	fn with_children_system(
		entity: Entity,
		children: impl FnOnce(&mut RelatedSpawnerCommands<ChildOf>) + Clone,
	) -> impl Fn(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);

			<EntityCommands as WithChildren>::with_children(&mut entity, children.clone());
		}
	}

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_component() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(try_insert_if_new_system(entity, _Component("Hey, there")))?;

		assert_eq!(
			Some(&_Component("Hey, there")),
			app.world().entity(entity).get::<_Component>(),
		);
		Ok(())
	}

	#[test]
	fn do_no_override_existing_component() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Component("don't forget me")).id();

		app.world_mut()
			.run_system_once(try_insert_if_new_system(entity, _Component("I don't care")))?;

		assert_eq!(
			Some(&_Component("don't forget me")),
			app.world().entity(entity).get::<_Component>(),
		);
		Ok(())
	}

	#[test]
	fn insert_child() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(with_child_system(entity, _Component("Hey, there")))?;

		let [child] = assert_count!(1, get_children!(app, entity));
		assert_eq!(Some(&_Component("Hey, there")), child.get::<_Component>());
		Ok(())
	}

	#[test]
	fn insert_children() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(with_children_system(entity, |p| {
				p.spawn(_Component("Here I am"));
				p.spawn(_Component("And here am I"));
			}))?;

		let [child_a, child_b] = assert_count!(2, get_children!(app, entity));
		assert_eq!(
			(
				Some(&_Component("Here I am")),
				Some(&_Component("And here am I")),
			),
			(child_a.get::<_Component>(), child_b.get::<_Component>(),)
		);
		Ok(())
	}
}
