use crate::components::player::Player;
use bevy::{
	ecs::system::{StaticSystemParam, SystemParam},
	prelude::*,
};
use common::{
	tools::action_key::miscellaneous::Miscellaneous,
	traits::{
		accessors::get::{GetContext, TryGetContextMut, ViewOf},
		handles_input::{GetInputState, InputState},
		handles_interactive::{Interactive, InteractiveState, SetInteractiveState},
		handles_physics::{Interactions, IterInteractions},
	},
};

impl Player {
	pub(crate) fn toggle_interactive<TInput, TPhysics, TInteractive>(
		mut interactive: StaticSystemParam<TInteractive>,
		input: StaticSystemParam<TInput>,
		physics: StaticSystemParam<TPhysics>,
		players: Query<Entity, With<Self>>,
	) where
		TInput: for<'w, 's> SystemParam<Item<'w, 's>: GetInputState>,
		TPhysics: for<'c> GetContext<Interactions, TContext<'c>: IterInteractions>,
		TInteractive: for<'c> TryGetContextMut<Interactive, TContext<'c>: SetInteractiveState>,
	{
		if input.get_input_state(Miscellaneous::Interact) != InputState::just_released() {
			return;
		}

		for entity in players {
			let key = Interactions { entity };
			let interactions = TPhysics::get_context(&physics, key);

			for entity in interactions.iter_interactions() {
				let key = Interactive { entity };
				let interactive = TInteractive::try_get_context_mut(&mut interactive, key);
				let Some(mut interactive) = interactive else {
					continue;
				};

				let state = match interactive.view_of::<InteractiveState>() {
					InteractiveState::Active => InteractiveState::Inactive,
					InteractiveState::Inactive => InteractiveState::Active,
				};

				interactive.set_interactive_state(state);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::action_key::{ActionKey, miscellaneous::Miscellaneous},
		traits::{
			accessors::get::View,
			handles_input::InputState,
			handles_interactive::InteractiveState,
			handles_map_generation::InteractiveType,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, predicate::eq};
	use std::{iter::Copied, slice::Iter};
	use test_case::test_case;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Resource, NestedMocks)]
	struct _Input {
		mock: Mock_Input,
	}

	#[automock]
	impl GetInputState for _Input {
		fn get_input_state<TAction>(&self, action: TAction) -> InputState
		where
			TAction: Into<ActionKey> + 'static,
		{
			self.mock.get_input_state(action)
		}
	}

	#[derive(Resource, Debug, PartialEq)]
	struct _PlayerInteractions(Vec<Entity>);

	impl _PlayerInteractions {
		fn from_entities(entities: impl Into<Vec<Entity>>) -> Self {
			Self(entities.into())
		}
	}

	impl IterInteractions for _PlayerInteractions {
		type TIter<'a>
			= Copied<Iter<'a, Entity>>
		where
			Self: 'a;

		fn iter_interactions(&self) -> Self::TIter<'_> {
			self.0.iter().copied()
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Interactive(InteractiveState);

	impl View<InteractiveType> for _Interactive {
		fn view(&self) -> InteractiveType {
			panic!("SHOULD NOT BE USED")
		}
	}

	impl View<InteractiveState> for _Interactive {
		fn view(&self) -> InteractiveState {
			self.0
		}
	}

	impl SetInteractiveState for _Interactive {
		fn set_interactive_state(&mut self, interactive_state: InteractiveState) {
			self.0 = interactive_state;
		}
	}

	fn setup(input: _Input) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(input);
		app.insert_resource(_PlayerInteractions::from_entities([]));
		app.add_systems(
			Update,
			Player::toggle_interactive::<
				Res<_Input>,
				Res<_PlayerInteractions>,
				Query<Mut<_Interactive>>,
			>,
		);

		app
	}

	#[test]
	fn toggle_active() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(Miscellaneous::Interact))
				.return_const(InputState::just_released());
		}));
		let entity = app
			.world_mut()
			.spawn(_Interactive(InteractiveState::Inactive))
			.id();
		app.world_mut().spawn(Player);
		app.insert_resource(_PlayerInteractions::from_entities([entity]));

		app.update();

		assert_eq!(
			Some(&_Interactive(InteractiveState::Active)),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}

	#[test]
	fn toggle_inactive() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(Miscellaneous::Interact))
				.return_const(InputState::just_released());
		}));
		let entity = app
			.world_mut()
			.spawn(_Interactive(InteractiveState::Active))
			.id();
		app.world_mut().spawn(Player);
		app.insert_resource(_PlayerInteractions::from_entities([entity]));

		app.update();

		assert_eq!(
			Some(&_Interactive(InteractiveState::Inactive)),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}

	#[test]
	fn do_nothing_when_not_interacting() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(Miscellaneous::Interact))
				.return_const(InputState::just_released());
		}));
		let entity = app
			.world_mut()
			.spawn(_Interactive(InteractiveState::Inactive))
			.id();
		app.world_mut().spawn(Player);
		app.insert_resource(_PlayerInteractions::from_entities([]));

		app.update();

		assert_eq!(
			Some(&_Interactive(InteractiveState::Inactive)),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}

	#[test]
	fn do_nothing_when_player_missing() {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(Miscellaneous::Interact))
				.return_const(InputState::just_released());
		}));
		let entity = app
			.world_mut()
			.spawn(_Interactive(InteractiveState::Inactive))
			.id();
		app.world_mut().spawn_empty();
		app.insert_resource(_PlayerInteractions::from_entities([entity]));

		app.update();

		assert_eq!(
			Some(&_Interactive(InteractiveState::Inactive)),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}

	#[test_case(InputState::released(); "released")]
	#[test_case(InputState::pressed(); "pressed")]
	#[test_case(InputState::just_pressed(); "just pressed")]
	fn do_nothing_when_interaction(input_state: InputState) {
		let mut app = setup(_Input::new().with_mock(|mock| {
			mock.expect_get_input_state()
				.with(eq(Miscellaneous::Interact))
				.return_const(input_state);
		}));
		let entity = app
			.world_mut()
			.spawn(_Interactive(InteractiveState::Inactive))
			.id();
		app.world_mut().spawn(Player);
		app.insert_resource(_PlayerInteractions::from_entities([entity]));

		app.update();

		assert_eq!(
			Some(&_Interactive(InteractiveState::Inactive)),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}
}
