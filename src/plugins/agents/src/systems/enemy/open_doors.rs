use crate::components::enemy::Enemy;
use bevy::{ecs::system::StaticSystemParam, prelude::*};
use common::traits::{
	accessors::get::{GetContext, TryGetContextMut, ViewOf},
	handles_interactive::{Interactive, InteractiveState, SetInteractiveState},
	handles_map_generation::InteractiveType,
	handles_physics::{Interactions, IterInteractions},
};

impl Enemy {
	pub(crate) fn open_doors<TPhysics, TInteractive>(
		mut interactive: StaticSystemParam<TInteractive>,
		physics: StaticSystemParam<TPhysics>,
		enemies: Query<Entity, With<Self>>,
	) where
		TPhysics: for<'c> GetContext<Interactions, TContext<'c>: IterInteractions>,
		TInteractive: for<'c> TryGetContextMut<Interactive, TContext<'c>: SetInteractiveState>,
	{
		for entity in enemies {
			let key = Interactions { entity };
			let interactions = TPhysics::get_context(&physics, key);

			for entity in interactions.iter_interactions() {
				let key = Interactive { entity };
				let interactive = TInteractive::try_get_context_mut(&mut interactive, key);
				let Some(mut interactive) = interactive else {
					continue;
				};

				if interactive.view_of::<InteractiveType>() != InteractiveType::Door {
					continue;
				}

				if interactive.view_of::<InteractiveState>() == InteractiveState::Active {
					continue;
				}

				interactive.set_interactive_state(InteractiveState::Active);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::Units,
		traits::{
			accessors::get::View,
			handles_interactive::InteractiveState,
			handles_map_generation::InteractiveType,
		},
	};
	use std::{iter::Copied, slice::Iter};
	use testing::{IsChanged, SingleThreadedApp};

	#[derive(Resource, Debug, PartialEq)]
	struct _EnemyInteractions(Vec<Entity>);

	impl _EnemyInteractions {
		fn from_entities(entities: impl Into<Vec<Entity>>) -> Self {
			Self(entities.into())
		}
	}

	impl IterInteractions for _EnemyInteractions {
		type TIter<'a>
			= Copied<Iter<'a, Entity>>
		where
			Self: 'a;

		fn iter_interactions(&self) -> Self::TIter<'_> {
			self.0.iter().copied()
		}
	}

	#[derive(Component, Debug, PartialEq)]
	struct _Interactive {
		of_type: InteractiveType,
		state: InteractiveState,
	}

	impl View<InteractiveType> for _Interactive {
		fn view(&self) -> InteractiveType {
			self.of_type
		}
	}

	impl View<InteractiveState> for _Interactive {
		fn view(&self) -> InteractiveState {
			self.state
		}
	}

	impl SetInteractiveState for _Interactive {
		fn set_interactive_state(&mut self, interactive_state: InteractiveState) {
			self.state = interactive_state;
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(_EnemyInteractions::from_entities([]));
		app.add_systems(
			Update,
			(
				Enemy::open_doors::<Res<_EnemyInteractions>, Query<Mut<_Interactive>>>,
				IsChanged::<_Interactive>::detect,
			)
				.chain(),
		);

		app
	}

	const ENEMY: Enemy = Enemy {
		aggro_range: Units::ZERO,
		attack_range: Units::ZERO,
		min_target_distance: None,
	};

	#[test]
	fn set_active() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_Interactive {
				state: InteractiveState::Inactive,
				of_type: InteractiveType::Door,
			})
			.id();
		app.world_mut().spawn(ENEMY);
		app.insert_resource(_EnemyInteractions::from_entities([entity]));

		app.update();

		assert_eq!(
			Some(&_Interactive {
				state: InteractiveState::Active,
				of_type: InteractiveType::Door
			}),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}

	#[test]
	fn do_nothing_if_not_interacting() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_Interactive {
				state: InteractiveState::Inactive,
				of_type: InteractiveType::Door,
			})
			.id();
		app.world_mut().spawn(ENEMY);
		app.insert_resource(_EnemyInteractions::from_entities([]));

		app.update();

		assert_eq!(
			Some(&_Interactive {
				state: InteractiveState::Inactive,
				of_type: InteractiveType::Door
			}),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}

	#[test]
	fn do_nothing_if_enemy_missing() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_Interactive {
				state: InteractiveState::Inactive,
				of_type: InteractiveType::Door,
			})
			.id();
		app.world_mut().spawn_empty();
		app.insert_resource(_EnemyInteractions::from_entities([entity]));

		app.update();

		assert_eq!(
			Some(&_Interactive {
				state: InteractiveState::Inactive,
				of_type: InteractiveType::Door
			}),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}

	#[test]
	fn do_nothing_if_interactive_is_not_door() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_Interactive {
				state: InteractiveState::Inactive,
				of_type: InteractiveType::Container,
			})
			.id();
		app.world_mut().spawn(ENEMY);
		app.insert_resource(_EnemyInteractions::from_entities([entity]));

		app.update();

		assert_eq!(
			Some(&_Interactive {
				state: InteractiveState::Inactive,
				of_type: InteractiveType::Container,
			}),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_Interactive {
				state: InteractiveState::Inactive,
				of_type: InteractiveType::Door,
			})
			.id();
		app.world_mut().spawn(ENEMY);
		app.insert_resource(_EnemyInteractions::from_entities([entity]));

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<_Interactive>>(),
		);
	}

	#[test]
	fn act_again_if_interactive_is_inactive() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(_Interactive {
				state: InteractiveState::Inactive,
				of_type: InteractiveType::Door,
			})
			.id();
		app.world_mut().spawn(ENEMY);
		app.insert_resource(_EnemyInteractions::from_entities([entity]));

		app.update();
		app.world_mut().entity_mut(entity).insert(_Interactive {
			state: InteractiveState::Inactive,
			of_type: InteractiveType::Door,
		});
		app.update();

		assert_eq!(
			Some(&_Interactive {
				state: InteractiveState::Active,
				of_type: InteractiveType::Door,
			}),
			app.world().entity(entity).get::<_Interactive>(),
		);
	}
}
