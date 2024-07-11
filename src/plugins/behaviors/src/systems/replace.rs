use bevy::ecs::{
	bundle::Bundle,
	component::Component,
	entity::Entity,
	system::{Commands, Query},
};
use common::traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom};

pub(crate) fn replace<TSource: Component + Clone, TTarget: Bundle + From<TSource>>(
	mut commands: Commands,
	sources: Query<(Entity, &TSource)>,
) {
	for (id, source) in &sources {
		commands.try_insert_on(id, TTarget::from(source.clone()));
		commands.try_remove_from::<TSource>(id);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
		utils::default,
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Source(usize);

	#[derive(Component, Debug, PartialEq)]
	struct _Target(usize);

	impl From<_Source> for _Target {
		fn from(source: _Source) -> Self {
			_Target(source.0)
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, replace::<_Source, _Target>);

		app
	}

	#[test]
	fn add_target() {
		let mut app = setup();

		let agent = app.world_mut().spawn(_Source(42)).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(Some(&_Target(42)), agent.get::<_Target>(),)
	}

	#[test]
	fn remove_source() {
		let mut app = setup();

		let agent = app.world_mut().spawn(_Source(default())).id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<_Source>(),)
	}
}
