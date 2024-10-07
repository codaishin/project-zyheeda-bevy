use crate::{components::Animator, AnimationData};
use bevy::prelude::*;
use common::traits::try_insert_on::TryInsertOn;

pub(crate) fn init_animation_components<TAgent: Component + Sync + Send + 'static>(
	mut commands: Commands,
	agents: Query<Entity, With<TAgent>>,
	animation_data: Res<AnimationData<TAgent>>,
	animation_players: Query<Entity, Added<AnimationPlayer>>,
	parents: Query<&Parent>,
) {
	for animation_player in &animation_players {
		for parent in parents.iter_ancestors(animation_player) {
			let Ok(agent) = agents.get(parent) else {
				continue;
			};
			commands.try_insert_on(agent, Animator { animation_player });
			commands.try_insert_on(
				animation_player,
				(
					animation_data.graph.clone(),
					AnimationTransitions::default(),
				),
			);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::animation_dispatch::AnimationDispatch;
	use bevy::prelude::{App, Update};
	use common::test_tools::utils::SingleThreadedApp;
	use uuid::Uuid;

	#[derive(Component)]
	struct _Agent;

	fn new_handle<T: Asset>() -> Handle<T> {
		Handle::Weak(AssetId::Uuid {
			uuid: Uuid::new_v4(),
		})
	}

	fn setup(animation_data: AnimationData<_Agent>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, init_animation_components::<_Agent>);
		app.insert_resource(animation_data);

		app
	}

	#[test]
	fn add_animator_with_animation_player_id() {
		let mut app = setup(AnimationData::new(new_handle()));
		let agent = app.world_mut().spawn(_Agent).id();
		let animation_player = app
			.world_mut()
			.spawn(AnimationPlayer::default())
			.set_parent(agent)
			.id();

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&Animator { animation_player }),
			agent.get::<Animator>()
		);
	}

	#[test]
	fn add_animation_graph() {
		let animation_graph_handle = new_handle();
		let mut app = setup(AnimationData::new(animation_graph_handle.clone()));
		let agent = app.world_mut().spawn(_Agent).id();
		let animation_player = app
			.world_mut()
			.spawn(AnimationPlayer::default())
			.set_parent(agent)
			.id();

		app.update();

		let animation_player = app.world().entity(animation_player);

		assert_eq!(
			Some(&animation_graph_handle),
			animation_player.get::<Handle<AnimationGraph>>()
		);
	}

	#[test]
	fn add_animation_transitions() {
		let mut app = setup(AnimationData::new(new_handle()));
		let agent = app.world_mut().spawn(_Agent).id();
		let animation_player = app
			.world_mut()
			.spawn(AnimationPlayer::default())
			.set_parent(agent)
			.id();

		app.update();

		let animation_player = app.world().entity(animation_player);

		assert!(animation_player.contains::<AnimationTransitions>());
	}

	#[test]
	fn add_components_only_once() {
		let mut app = setup(AnimationData::new(new_handle()));
		let agent = app.world_mut().spawn(_Agent).id();
		let animation_player = app
			.world_mut()
			.spawn(AnimationPlayer::default())
			.set_parent(agent)
			.id();

		app.update();

		let mut entity = app.world_mut().entity_mut(agent);
		entity.remove::<Animator>();

		let mut entity = app.world_mut().entity_mut(animation_player);
		entity.remove::<Handle<AnimationGraph>>();
		entity.remove::<AnimationTransitions>();

		app.update();

		let agent = app.world().entity(agent);
		let animation_player = app.world().entity(animation_player);

		assert_eq!(
			(false, false, false),
			(
				agent.contains::<Animator>(),
				animation_player.contains::<Handle<AnimationGraph>>(),
				animation_player.contains::<AnimationTransitions>()
			)
		);
	}

	#[test]
	fn add_none_when_agent_component_not_parent() {
		let mut app = setup(AnimationData::new(new_handle()));
		let agent = app.world_mut().spawn_empty().id();
		let animation_player = app
			.world_mut()
			.spawn(AnimationPlayer::default())
			.set_parent(agent)
			.id();

		app.update();

		let agent = app.world().entity(agent);
		let animation_player = app.world().entity(animation_player);

		assert_eq!(
			(false, false, false, false),
			(
				agent.contains::<Animator>(),
				agent.contains::<AnimationDispatch>(),
				animation_player.contains::<Handle<AnimationGraph>>(),
				animation_player.contains::<AnimationTransitions>()
			)
		);
	}
}
