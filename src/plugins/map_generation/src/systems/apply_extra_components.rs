use crate::traits::Definition;
use bevy::{
	core::Name,
	ecs::{
		entity::Entity,
		query::Added,
		system::{Commands, Query},
	},
};

pub(crate) fn apply_extra_components<TDefinition: Definition>(
	mut commands: Commands,
	new: Query<(Entity, &Name), Added<Name>>,
) {
	if new.is_empty() {
		return;
	}

	let target_names = TDefinition::target_names();

	for (id, ..) in new.iter().filter(contained_in(target_names)) {
		let Some(entity) = &mut commands.get_entity(id) else {
			continue;
		};
		TDefinition::insert_bundle(entity);
	}
}

fn contained_in(target_names: Vec<String>) -> impl Fn(&(Entity, &Name)) -> bool {
	move |(.., name)| target_names.contains(&name.as_str().to_owned())
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::{component::Component, system::EntityCommands},
	};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::automock;

	struct _Definition;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Component;

	impl Definition for _Definition {
		fn target_names() -> Vec<String> {
			vec!["AAA".to_owned()]
		}

		fn insert_bundle(entity: &mut EntityCommands) {
			entity.insert(_Component);
		}
	}

	fn setup<TDefinition: Definition + 'static>() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, apply_extra_components::<TDefinition>);

		app
	}

	#[test]
	fn add_component_when_name_matches() {
		let mut app = setup::<_Definition>();
		let agent = app.world.spawn(Name::new("AAA")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Component), agent.get::<_Component>());
	}

	#[test]
	fn ignore_when_name_not_matching() {
		let mut app = setup::<_Definition>();
		let agent = app.world.spawn(Name::new("CCC")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_Component>());
	}

	#[test]
	fn do_only_operate_once() {
		let mut app = setup::<_Definition>();
		let agent = app.world.spawn(Name::new("AAA")).id();

		app.update();

		app.world.entity_mut(agent).remove::<_Component>();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_Component>());
	}

	struct _Definition2;

	#[automock]
	impl Definition for _Definition2 {
		fn target_names() -> Vec<String> {
			todo!()
		}

		#[allow(clippy::needless_lifetimes)]
		fn insert_bundle<'a>(_entity: &mut EntityCommands<'a>) {
			todo!()
		}
	}

	#[test]
	fn do_not_call_target_names_multiple_times() {
		let mut app = setup::<Mock_Definition2>();
		app.world.spawn(Name::new("AAA"));
		app.world.spawn(Name::new("AAA"));

		let target_names = Mock_Definition2::target_names_context();
		target_names.expect().times(1).return_const(vec![]);

		app.update();
	}
}
