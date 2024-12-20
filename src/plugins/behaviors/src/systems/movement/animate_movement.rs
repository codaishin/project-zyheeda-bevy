use crate::traits::{GetAnimation, MovementData};
use bevy::ecs::{
	change_detection::DetectChanges,
	component::Component,
	entity::Entity,
	query::Without,
	removal_detection::RemovedComponents,
	system::Query,
	world::Ref,
};
use common::{
	components::Immobilized,
	traits::animation::{AnimationPriority, StartAnimation, StopAnimation},
};

type Components<'a, TMovementConfig, TAnimations, TAnimationDispatch, TMovement> = (
	Ref<'a, TMovementConfig>,
	&'a TAnimations,
	&'a mut TAnimationDispatch,
	Ref<'a, TMovement>,
);

#[derive(Debug, PartialEq)]
struct Move;

impl From<Move> for AnimationPriority {
	fn from(_: Move) -> Self {
		AnimationPriority::Medium
	}
}

pub(crate) fn animate_movement<
	TMovementConfig: Component + MovementData,
	TMovement: Component,
	TAnimations: Component + GetAnimation,
	TAnimationDispatch: Component + StartAnimation + StopAnimation,
>(
	mut agents: Query<
		Components<TMovementConfig, TAnimations, TAnimationDispatch, TMovement>,
		Without<Immobilized>,
	>,
	mut agents_without_movement: Query<&mut TAnimationDispatch, Without<TMovement>>,
	mut removed_movements: RemovedComponents<TMovement>,
) {
	for (config, animations, dispatch, movement) in &mut agents {
		start_animation(config, animations, dispatch, movement);
	}

	for entity in removed_movements.read() {
		stop_animation(entity, &mut agents_without_movement);
	}
}

fn start_animation<
	TMovementConfig: MovementData,
	TMovement,
	TAnimations: GetAnimation,
	TAnimationDispatch: StartAnimation,
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
	dispatch.start_animation(Move, animation.clone());
}

fn stop_animation<TMovement: Component, TAnimationDispatch: Component + StopAnimation>(
	entity: Entity,
	agent_without_movement: &mut Query<&mut TAnimationDispatch, Without<TMovement>>,
) {
	let Ok(mut dispatch) = agent_without_movement.get_mut(entity) else {
		return;
	};
	dispatch.stop_animation(Move);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::MovementMode;
	use bevy::{
		app::{App, Update},
		utils::default,
	};
	use common::{
		test_tools::utils::SingleThreadedApp,
		tools::UnitsPerSecond,
		traits::{
			animation::{Animation, PlayMode},
			load_asset::Path,
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{automock, mock, predicate::eq};
	use std::ops::DerefMut;

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

	#[derive(Component, NestedMocks)]
	struct _MovementAnimations {
		mock: Mock_MovementAnimations,
	}

	impl Default for _MovementAnimations {
		fn default() -> Self {
			Self::new().with_mock(|mock| {
				mock.expect_animation()
					.return_const(Animation::new(Path::from(""), PlayMode::Repeat));
			})
		}
	}

	#[automock]
	impl GetAnimation for _MovementAnimations {
		fn animation(&self, key: &MovementMode) -> &Animation {
			self.mock.animation(key)
		}
	}

	#[derive(Component, NestedMocks)]
	struct _AnimationDispatch {
		mock: Mock_AnimationDispatch,
	}

	impl StartAnimation for _AnimationDispatch {
		fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
		where
			TLayer: Into<AnimationPriority> + 'static,
		{
			self.mock.start_animation(layer, animation)
		}
	}

	impl StopAnimation for _AnimationDispatch {
		fn stop_animation<TLayer>(&mut self, layer: TLayer)
		where
			TLayer: Into<AnimationPriority> + 'static,
		{
			self.mock.stop_animation(layer)
		}
	}

	mock! {
		_AnimationDispatch {}
		impl StartAnimation for _AnimationDispatch {
			fn start_animation<TLayer>(&mut self, layer: TLayer, animation: Animation)
				where TLayer: Into<AnimationPriority> + 'static;
		}
		impl StopAnimation for _AnimationDispatch {
			fn stop_animation<TLayer>(&mut self, layer: TLayer)
				where TLayer: Into<AnimationPriority> + 'static;
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			animate_movement::<_Config, _Movement, _MovementAnimations, _AnimationDispatch>,
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
					.return_const(Animation::new(Path::from("fast"), PlayMode::Repeat));
			}),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation()
					.times(1)
					.with(
						eq(Move),
						eq(Animation::new(Path::from("fast"), PlayMode::Repeat)),
					)
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
					.return_const(Animation::new(Path::from("slow"), PlayMode::Repeat));
			}),
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation()
					.times(1)
					.with(
						eq(Move),
						eq(Animation::new(Path::from("slow"), PlayMode::Repeat)),
					)
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
				mock.expect_start_animation::<Move>()
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
					mock.expect_start_animation::<Move>().return_const(());
					mock.expect_stop_animation::<Move>()
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
				mock.expect_start_animation::<Move>()
					.times(1)
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
					mock.expect_start_animation::<Move>()
						.times(1)
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
					mock.expect_start_animation::<Move>()
						.times(2)
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
				mock.expect_start_animation::<Move>()
					.never()
					.return_const(());
			}),
			_Movement,
			Immobilized,
		));
		app.update();
	}
}
