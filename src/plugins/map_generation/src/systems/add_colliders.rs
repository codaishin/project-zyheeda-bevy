use crate::traits::ColliderDefinition;
use bevy::{
	core::Name,
	ecs::{
		entity::Entity,
		query::Added,
		system::{Commands, EntityCommands, Query},
	},
};
use common::components::NoTarget;

pub(crate) fn add_colliders<TDefinition: ColliderDefinition>(
	mut commands: Commands,
	new: Query<(Entity, &Name), Added<Name>>,
) {
	if new.is_empty() {
		return;
	}

	let insert_fn = match TDefinition::IS_TARGET {
		true => insert_collider::<TDefinition>,
		false => insert_non_target_able_collider::<TDefinition>,
	};

	for (id, ..) in new.iter().filter(contained_in(TDefinition::target_names())) {
		let Some(entity) = commands.get_entity(id) else {
			continue;
		};
		insert_fn(entity);
	}
}

fn insert_collider<TDefinition: ColliderDefinition>(mut entity: EntityCommands) {
	entity.try_insert(TDefinition::collider());
}
fn insert_non_target_able_collider<TDefinition: ColliderDefinition>(mut entity: EntityCommands) {
	entity.try_insert((TDefinition::collider(), NoTarget));
}

fn contained_in(target_names: Vec<String>) -> impl Fn(&(Entity, &Name)) -> bool {
	move |(.., name)| target_names.contains(&name.as_str().to_owned())
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::app::{App, Update};
	use bevy_rapier3d::geometry::Collider;
	use common::{components::NoTarget, test_tools::utils::SingleThreadedApp};
	use mockall::automock;

	struct _Definition<const IS_TARGET: bool>;

	impl<const IS_TARGET: bool> ColliderDefinition for _Definition<IS_TARGET> {
		const IS_TARGET: bool = IS_TARGET;

		fn target_names() -> Vec<String> {
			vec!["AAA".to_owned()]
		}

		fn collider() -> Collider {
			Collider::ball(42.)
		}
	}

	fn setup<TDefinition: ColliderDefinition + 'static>() -> App {
		let mut app = App::new_single_threaded([Update]);
		app.add_systems(Update, add_colliders::<TDefinition>);

		app
	}

	#[test]
	fn add_collider() {
		let mut app = setup::<_Definition<false>>();
		let agent = app.world.spawn(Name::new("AAA")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(Collider::ball(42.).raw.as_ball()),
			agent.get::<Collider>().map(|c| c.raw.as_ball())
		);
	}

	#[test]
	fn add_no_target() {
		let mut app = setup::<_Definition<false>>();
		let agent = app.world.spawn(Name::new("AAA")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(Some(&NoTarget), agent.get::<NoTarget>());
	}

	#[test]
	fn add_non_target_able() {
		let mut app = setup::<_Definition<true>>();
		let agent = app.world.spawn(Name::new("AAA")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(Some(Collider::ball(42.).raw.as_ball()), None),
			(
				agent.get::<Collider>().map(|c| c.raw.as_ball()),
				agent.get::<NoTarget>()
			)
		);
	}

	#[test]
	fn ignore_when_name_not_matching() {
		let mut app = setup::<_Definition<false>>();
		let agent = app.world.spawn(Name::new("CCC")).id();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(None, None),
			(
				agent.get::<Collider>().map(|c| c.raw.as_ball()),
				agent.get::<NoTarget>()
			)
		);
	}

	#[test]
	fn do_only_operate_once() {
		let mut app = setup::<_Definition<false>>();
		let agent = app.world.spawn(Name::new("AAA")).id();

		app.update();

		app.world.entity_mut(agent).remove::<(NoTarget, Collider)>();

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(None, None),
			(
				agent.get::<Collider>().map(|c| c.raw.as_ball()),
				agent.get::<NoTarget>()
			)
		);
	}

	struct _Definition2;

	#[automock]
	impl ColliderDefinition for _Definition2 {
		const IS_TARGET: bool = false;

		fn target_names() -> Vec<String> {
			todo!()
		}

		fn collider() -> Collider {
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
