use crate::traits::AnimationPlayersWithoutGraph;
use bevy::prelude::*;
use common::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};

impl<T> InitPlayerComponents for T where for<'a> T: Component + AnimationPlayersWithoutGraph + Sized {}

pub(crate) trait InitPlayerComponents:
	Component + AnimationPlayersWithoutGraph + Sized
{
	fn init_player_components<TGraphComponent>(
		mut commands: ZyheedaCommands,
		agents: Query<(&Self, &TGraphComponent), Changed<Self>>,
	) where
		TGraphComponent: Component + Clone,
	{
		for (dispatcher, graph_component) in &agents {
			for entity in dispatcher.animation_players_without_graph() {
				commands.try_apply_on(&entity, |mut e| {
					e.try_insert((AnimationTransitions::default(), graph_component.clone()));
				});
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::vec::IntoIter;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Dispatch {
		players: Vec<Entity>,
	}

	impl AnimationPlayersWithoutGraph for _Dispatch {
		type TIter = IntoIter<Entity>;

		fn animation_players_without_graph(&self) -> Self::TIter {
			self.players.clone().into_iter()
		}
	}

	#[derive(Component, Debug, PartialEq, Clone)]
	struct _GraphComponent;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, _Dispatch::init_player_components::<_GraphComponent>);

		app
	}

	#[test]
	fn add_transitions() {
		let mut app = setup();
		let player = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			_Dispatch {
				players: vec![player],
			},
			_GraphComponent,
		));

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
		let player = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			_Dispatch {
				players: vec![player],
			},
			_GraphComponent,
		));

		app.update();

		assert!(app.world().entity(player).contains::<_GraphComponent>());
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let player = app.world_mut().spawn_empty().id();
		app.world_mut().spawn((
			_Dispatch {
				players: vec![player],
			},
			_GraphComponent,
		));

		app.update();
		app.world_mut()
			.entity_mut(player)
			.remove::<(AnimationTransitions, _GraphComponent)>();
		app.update();

		assert_eq!(
			[false, false],
			[
				app.world()
					.entity(player)
					.contains::<AnimationTransitions>(),
				app.world().entity(player).contains::<_GraphComponent>(),
			]
		);
	}

	#[test]
	fn act_again_when_dispatch_changed() {
		let mut app = setup();
		let player = app.world_mut().spawn_empty().id();
		let dispatch = app
			.world_mut()
			.spawn((
				_Dispatch {
					players: vec![player],
				},
				_GraphComponent,
			))
			.id();

		app.update();
		app.world_mut()
			.entity_mut(player)
			.remove::<(AnimationTransitions, AnimationGraphHandle)>();
		app.world_mut()
			.entity_mut(dispatch)
			.get_mut::<_Dispatch>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			[true, true],
			[
				app.world()
					.entity(player)
					.contains::<AnimationTransitions>(),
				app.world().entity(player).contains::<_GraphComponent>(),
			]
		);
	}
}
