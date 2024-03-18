use crate::traits::Definition;
use bevy::{
	core::Name,
	ecs::{
		bundle::Bundle,
		entity::Entity,
		query::Added,
		system::{Commands, Query},
	},
	hierarchy::Children,
};
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn add_component<TDefinition: Definition<TBundle>, TBundle: Bundle + Clone>(
	mut commands: Commands,
	new: Query<(Entity, &Name, Option<&Children>), Added<Name>>,
) {
	if new.is_empty() {
		return;
	}

	let target_names = TDefinition::target_names();

	for (id, _, children) in new.iter().filter(contained_in(target_names)) {
		let (bundle, for_children) = TDefinition::bundle();
		match (*for_children, children) {
			(true, Some(children)) => try_insert_on_children(&mut commands, children, bundle),
			(false, _) => try_insert(&mut commands, id, bundle),
			_ => {}
		};
	}
}

fn contained_in(target_names: Vec<String>) -> impl Fn(&(Entity, &Name, Option<&Children>)) -> bool {
	move |(_, name, _)| target_names.contains(&name.as_str().to_owned())
}

fn try_insert<TBundle: Bundle + Clone>(commands: &mut Commands, id: Entity, bundle: TBundle) {
	commands.try_insert_on(id, bundle)
}

fn try_insert_on_children<TBundle: Bundle + Clone>(
	commands: &mut Commands,
	children: &Children,
	bundle: TBundle,
) {
	for child in children {
		commands.try_insert_on(*child, bundle.clone());
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::types::ForChildren;
	use bevy::{
		app::{App, Update},
		ecs::component::Component,
		hierarchy::BuildWorldChildren,
	};
	use common::test_tools::utils::SingleThreadedApp;
	use mockall::automock;

	struct _Definition<const FOR_CHILDREN: bool>;

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _Component;

	impl<const FOR_CHILDREN: bool> Definition<_Component> for _Definition<FOR_CHILDREN> {
		fn target_names() -> Vec<String> {
			vec!["AAA".to_owned()]
		}

		fn bundle() -> (_Component, ForChildren) {
			(_Component, FOR_CHILDREN.into())
		}
	}

	fn setup<TDefinition: Definition<_Component> + 'static>() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, add_component::<TDefinition, _Component>);

		app
	}

	#[test]
	fn add_component_when_name_matches() {
		let mut app = setup::<_Definition<false>>();
		let agent = app.world.spawn(Name::new("AAA")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&_Component), agent.get::<_Component>());
	}

	#[test]
	fn ignore_when_name_not_matching() {
		let mut app = setup::<_Definition<false>>();
		let agent = app.world.spawn(Name::new("CCC")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_Component>());
	}

	#[test]
	fn add_component_when_name_matches_on_children() {
		let mut app = setup::<_Definition<true>>();
		let agent = app.world.spawn(Name::new("AAA")).id();
		let children = [
			app.world.spawn_empty().set_parent(agent).id(),
			app.world.spawn_empty().set_parent(agent).id(),
			app.world.spawn_empty().set_parent(agent).id(),
		];

		app.update();

		let children = children.map(|child| app.world.entity(child));

		assert_eq!(
			[Some(&_Component), Some(&_Component), Some(&_Component)],
			children.map(|child| child.get::<_Component>())
		);
	}

	#[test]
	fn ignore_when_for_children_but_no_children_are_present() {
		let mut app = setup::<_Definition<true>>();
		let agent = app.world.spawn(Name::new("AAA")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_Component>());
	}

	#[test]
	fn do_only_operate_once() {
		let mut app = setup::<_Definition<false>>();
		let agent = app.world.spawn(Name::new("AAA")).id();

		app.update();

		app.world.entity_mut(agent).remove::<_Component>();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<_Component>());
	}

	struct _Definition2;

	#[automock]
	impl Definition<_Component> for _Definition2 {
		fn target_names() -> Vec<String> {
			todo!()
		}

		fn bundle() -> (_Component, ForChildren) {
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
