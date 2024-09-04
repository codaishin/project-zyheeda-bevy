use crate::traits::{GetAnimation, MovementData};
use animations::traits::{MovementLayer, StartAnimation, StopAnimation};
use bevy::ecs::{
	change_detection::DetectChanges,
	component::Component,
	entity::Entity,
	query::Without,
	removal_detection::RemovedComponents,
	system::Query,
	world::Ref,
};
use common::components::Immobilized;

type Components<'a, TMovementConfig, TAnimations, TAnimationDispatch, TMovement> = (
	Ref<'a, TMovementConfig>,
	&'a TAnimations,
	&'a mut TAnimationDispatch,
	Ref<'a, TMovement>,
);

pub(crate) fn animate_movement<
	TMovementConfig: Component + MovementData,
	TMovement: Component,
	TAnimation: Clone + Sync + Send + 'static,
	TAnimations: Component + GetAnimation<TAnimation>,
	TAnimationDispatch: Component + StartAnimation<TAnimation> + StopAnimation,
>(
	mut agents: Query<
		Components<TMovementConfig, TAnimations, TAnimationDispatch, TMovement>,
		Without<Immobilized>,
	>,
	mut agents_without_movement: Query<&mut TAnimationDispatch, Without<TMovement>>,
	mut removed_movements: RemovedComponents<TMovement>,
) {
	for (config, animations, dispatch, movement) in &mut agents {
		insert_animation(config, animations, dispatch, movement);
	}

	for entity in removed_movements.read() {
		remove_animation(entity, &mut agents_without_movement);
	}
}

fn insert_animation<
	TMovementConfig: MovementData,
	TMovement,
	TAnimation: Clone,
	TAnimations: GetAnimation<TAnimation>,
	TAnimationDispatch: StartAnimation<TAnimation>,
>(
	config: Ref<TMovementConfig>,
	animations: &TAnimations,
	mut dispatch: bevy::prelude::Mut<TAnimationDispatch>,
	movement: Ref<TMovement>,
) {
	if !movement.is_added() && !config.is_changed() {
		return;
	}
	let (.., mode) = config.get_movement_data();
	let animation = animations.animation(&mode);
	dispatch.start_animation(MovementLayer, animation.clone());
}

fn remove_animation<TMovement: Component, TAnimationDispatch: Component + StopAnimation>(
	entity: Entity,
	agent_without_movement: &mut Query<&mut TAnimationDispatch, Without<TMovement>>,
) {
	let Ok(mut dispatch) = agent_without_movement.get_mut(entity) else {
		return;
	};
	dispatch.stop_animation(MovementLayer);
}

#[cfg(test)]
mod tests {
	use std::ops::DerefMut;

	use super::*;
	use crate::components::MovementMode;
	use animations::traits::Priority;
	use bevy::{
		app::{App, Update},
		utils::default,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::nested_mock::NestedMocks,
	};
	use macros::NestedMocks;
	use mockall::{automock, mock, predicate::eq};

	#[derive(Component, NestedMocks)]
	struct _Config {
		mock: Mock_Config,
	}

	impl Default for _Config {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_get_movement_data()
					.return_const((default(), default()));
			})
		}
	}

	#[automock]
	impl MovementData for _Config {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			self.mock.get_movement_data()
		}
	}

	#[derive(Component)]
	struct _Movement;

	#[derive(Debug, PartialEq, Clone)]
	struct _Animation;

	#[derive(Component, NestedMocks)]
	struct _MovementAnimations {
		mock: Mock_MovementAnimations,
	}

	impl Default for _MovementAnimations {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_animation().return_const(_Animation);
			})
		}
	}

	#[automock]
	impl GetAnimation<_Animation> for _MovementAnimations {
		fn animation(&self, key: &MovementMode) -> &_Animation {
			self.mock.animation(key)
		}
	}

	#[derive(Component, NestedMocks)]
	struct _AnimationDispatch {
		mock: Mock_AnimationDispatch,
	}

	impl StartAnimation<_Animation> for _AnimationDispatch {
		fn start_animation<TLayer>(&mut self, layer: TLayer, animation: _Animation)
		where
			TLayer: 'static,
			Priority: From<TLayer>,
		{
			self.mock.start_animation(layer, animation)
		}
	}

	impl StopAnimation for _AnimationDispatch {
		fn stop_animation<TLayer>(&mut self, layer: TLayer)
		where
			TLayer: 'static,
			Priority: From<TLayer>,
		{
			self.mock.stop_animation(layer)
		}
	}

	mock! {
		_AnimationDispatch {}
		impl StartAnimation<_Animation> for _AnimationDispatch {
			fn start_animation<TLayer>(&mut self, layer: TLayer, animation: _Animation) where
				TLayer: 'static,
				Priority: From<TLayer>;
		}
		impl StopAnimation for _AnimationDispatch {
			fn stop_animation<TLayer>(&mut self, layer: TLayer)	where
				TLayer: 'static,
				Priority: From<TLayer>;
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			animate_movement::<
				_Config,
				_Movement,
				_Animation,
				_MovementAnimations,
				_AnimationDispatch,
			>,
		);

		app
	}

	#[test]
	fn animate_fast() {
		let mut app = setup();
		app.world_mut().spawn((
			_Config::new().with_mock(|mock| {
				mock.expect_get_movement_data()
					.return_const((UnitsPerSecond::default(), MovementMode::Fast));
			}),
			_MovementAnimations::new().with_mock(|mock| {
				mock.expect_animation()
					.times(1)
					.with(eq(MovementMode::Fast))
					.return_const(_Animation);
			}),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation()
					.times(1)
					.with(eq(MovementLayer), eq(_Animation))
					.return_const(());
			}),
			_Movement,
		));

		app.update();
	}

	#[test]
	fn animate_slow() {
		let mut app = setup();
		app.world_mut().spawn((
			_Config::new().with_mock(|mock| {
				mock.expect_get_movement_data()
					.return_const((UnitsPerSecond::default(), MovementMode::Slow));
			}),
			_MovementAnimations::new().with_mock(|mock| {
				mock.expect_animation()
					.times(1)
					.with(eq(MovementMode::Slow))
					.return_const(_Animation);
			}),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation()
					.times(1)
					.with(eq(MovementLayer), eq(_Animation))
					.return_const(());
			}),
			_Movement,
		));

		app.update();
	}

	#[test]
	fn do_not_animate_when_no_movement_component() {
		let mut app = setup();
		app.world_mut().spawn((
			_Config::default(),
			_MovementAnimations::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation::<MovementLayer>()
					.never()
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn remove_medium_priority_when_movement_removed() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Config::default(),
				_MovementAnimations::default(),
				_AnimationDispatch::new().with_mock(|mock| {
					mock.expect_start_animation::<MovementLayer>()
						.return_const(());
					mock.expect_stop_animation::<MovementLayer>()
						.times(1)
						.return_const(());
				}),
				_Movement,
			))
			.id();

		app.update();

		app.world_mut().entity_mut(agent).remove::<_Movement>();

		app.update();
	}

	#[test]
	fn animate_only_when_movement_added() {
		let mut app = setup();
		app.world_mut().spawn((
			_Config::default(),
			_MovementAnimations::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation()
					.times(1)
					.with(eq(MovementLayer), eq(_Animation))
					.return_const(());
			}),
			_Movement,
		));

		app.update();
		app.update();
	}

	#[test]
	fn animate_only_when_movement_added_and_not_mutable_dereferenced() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Config::default(),
				_MovementAnimations::default(),
				_AnimationDispatch::new().with_mock(|mock| {
					mock.expect_start_animation()
						.times(1)
						.with(eq(MovementLayer), eq(_Animation))
						.return_const(());
				}),
				_Movement,
			))
			.id();

		app.update();

		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_Movement>()
			.unwrap()
			.deref_mut();

		app.update();
	}

	#[test]
	fn animate_again_when_config_mutably_dereferenced() {
		let mut app = setup();
		let agent = app
			.world_mut()
			.spawn((
				_Config::default(),
				_MovementAnimations::default(),
				_AnimationDispatch::new().with_mock(|mock| {
					mock.expect_start_animation()
						.times(2)
						.with(eq(MovementLayer), eq(_Animation))
						.return_const(());
				}),
				_Movement,
			))
			.id();
		app.update();

		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_Config>()
			.unwrap()
			.deref_mut();

		app.update();
	}

	#[test]
	fn no_animate_when_immobilized() {
		let mut app = setup();
		app.world_mut().spawn((
			_Config::default(),
			_MovementAnimations::default(),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation::<MovementLayer>()
					.never()
					.return_const(());
			}),
			_Movement,
			Immobilized,
		));
		app.update();
	}
}
