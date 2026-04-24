use crate::components::animation_dispatch::{AnimationDispatch, AnimationPlayers};
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl AnimationDispatch {
	pub(crate) fn distribute_player_components(
		mut commands: ZyheedaCommands,
		agents: Query<(&AnimationPlayers, &AnimationGraphHandle), Changed<AnimationPlayers>>,
	) {
		for (animation_players, graph_component) in &agents {
			for entity in animation_players.iter() {
				commands.try_apply_on(&entity, |mut e| {
					e.try_insert((AnimationTransitions::default(), graph_component.clone()));
				});
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::components::animation_dispatch::AnimationPlayerOf;

	use super::*;
	use testing::{SingleThreadedApp, new_handle};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, AnimationDispatch::distribute_player_components);

		app
	}

	#[test]
	fn add_transitions() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(AnimationGraphHandle(new_handle()))
			.id();
		let player = app.world_mut().spawn(AnimationPlayerOf(agent)).id();

		app.update();

		assert!(
			app.world()
				.entity(player)
				.contains::<AnimationTransitions>()
		);
	}

	#[test]
	fn clone_graph_component() {
		let mut app = setup();
		let handle = new_handle();
		let agent = app
			.world_mut()
			.spawn(AnimationGraphHandle(handle.clone()))
			.id();
		let player = app.world_mut().spawn(AnimationPlayerOf(agent)).id();

		app.update();

		assert_eq!(
			Some(&AnimationGraphHandle(handle)),
			app.world().entity(player).get::<AnimationGraphHandle>()
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(AnimationGraphHandle(new_handle()))
			.id();
		let player = app.world_mut().spawn(AnimationPlayerOf(agent)).id();

		app.update();
		app.world_mut()
			.entity_mut(player)
			.remove::<(AnimationTransitions, AnimationGraphHandle)>();
		app.update();

		assert_eq!(
			[false, false],
			[
				app.world()
					.entity(player)
					.contains::<AnimationTransitions>(),
				app.world()
					.entity(player)
					.contains::<AnimationGraphHandle>(),
			]
		);
	}

	#[test]
	fn act_again_when_animation_players_changed() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn(AnimationGraphHandle(new_handle()))
			.id();
		let player = app.world_mut().spawn(AnimationPlayerOf(agent)).id();

		app.update();
		app.world_mut()
			.entity_mut(player)
			.remove::<(AnimationTransitions, AnimationGraphHandle)>();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<AnimationPlayers>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			[true, true],
			[
				app.world()
					.entity(player)
					.contains::<AnimationTransitions>(),
				app.world()
					.entity(player)
					.contains::<AnimationGraphHandle>(),
			]
		);
	}
}
