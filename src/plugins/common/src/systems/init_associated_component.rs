use crate::traits::try_insert_on::TryInsertOn;
use bevy::prelude::*;

impl<TComponent> InitAssociatedComponent for TComponent where Self: Component {}

pub trait InitAssociatedComponent
where
	Self: Component,
{
	fn init_associated<TComponent>(mut commands: Commands, agents: Query<Entity, Added<Self>>)
	where
		Self: GetAssociated<TComponent> + Component + Sized,
		TComponent: Component,
	{
		for entity in &agents {
			commands.try_insert_on(entity, Self::get_associated_component());
		}
	}
}

pub trait GetAssociated<TComponent>
where
	TComponent: Component,
{
	fn get_associated_component() -> TComponent;
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq)]
	struct _Associated;

	impl GetAssociated<_Associated> for _Agent {
		fn get_associated_component() -> _Associated {
			_Associated
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Agent::init_associated::<_Associated>);

		app
	}

	#[test]
	fn add_associated_component() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Agent).id();

		app.update();

		assert_eq!(
			Some(&_Associated),
			app.world().entity(entity).get::<_Associated>(),
		)
	}

	#[test]
	fn do_not_add_associated_component_when_no_agent() {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Associated>(),)
	}

	#[test]
	fn do_not_add_associated_component_when_agent_not_new() {
		let mut app = setup();
		let entity = app.world_mut().spawn(_Agent).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<_Associated>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Associated>(),)
	}
}
