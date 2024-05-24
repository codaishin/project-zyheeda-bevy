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
	TAnimationDispatch: Component + StartAnimation<MovementLayer, TAnimation> + StopAnimation<MovementLayer>,
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
	TAnimationDispatch: StartAnimation<MovementLayer, TAnimation>,
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
	dispatch.start_animation(animation.clone());
}

fn remove_animation<
	TMovement: Component,
	TAnimationDispatch: Component + StopAnimation<MovementLayer>,
>(
	entity: Entity,
	agent_without_movement: &mut Query<&mut TAnimationDispatch, Without<TMovement>>,
) {
	let Ok(mut dispatch) = agent_without_movement.get_mut(entity) else {
		return;
	};
	dispatch.stop_animation();
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::MovementMode;
	use bevy::app::{App, Update};
	use common::{test_tools::utils::SingleThreadedApp, tools::UnitsPerSecond};
	use mockall::{automock, mock, predicate::eq};

	#[derive(Component, Default)]
	struct _Config {
		mock: Mock_Config,
	}

	#[automock]
	impl MovementData for _Config {
		fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
			self.mock.get_movement_data()
		}
	}

	#[derive(Component, Default)]
	struct _Movement(&'static str);

	#[derive(Debug, PartialEq, Clone)]
	struct _Animation(&'static str);

	#[derive(Component, Default)]
	struct _MovementAnimations {
		mock: Mock_MovementAnimations,
	}

	#[automock]
	impl GetAnimation<_Animation> for _MovementAnimations {
		fn animation(&self, key: &MovementMode) -> &_Animation {
			self.mock.animation(key)
		}
	}

	#[derive(Component, Default)]
	struct _AnimationDispatch {
		mock: Mock_AnimationDispatch,
	}

	impl StartAnimation<MovementLayer, _Animation> for _AnimationDispatch {
		fn start_animation(&mut self, animation: _Animation) {
			self.mock.start_animation(animation)
		}
	}

	impl StopAnimation<MovementLayer> for _AnimationDispatch {
		fn stop_animation(&mut self) {
			self.mock.stop_animation()
		}
	}

	mock! {
		_AnimationDispatch {}
		impl StartAnimation<MovementLayer, _Animation> for _AnimationDispatch {
			fn start_animation(&mut self, animation: _Animation);
		}
		impl StopAnimation<MovementLayer> for _AnimationDispatch {
			fn stop_animation(&mut self);
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
		let mut config = _Config::default();
		let mut animations = _MovementAnimations::default();
		let mut dispatch = _AnimationDispatch::default();

		config
			.mock
			.expect_get_movement_data()
			.return_const((UnitsPerSecond::default(), MovementMode::Fast));
		animations
			.mock
			.expect_animation()
			.with(eq(MovementMode::Fast))
			.return_const(_Animation("fast"));
		animations
			.mock
			.expect_animation()
			.with(eq(MovementMode::Slow))
			.return_const(_Animation("slow"));

		dispatch
			.mock
			.expect_start_animation()
			.times(1)
			.with(eq(_Animation("fast")))
			.return_const(());

		app.world
			.spawn((config, animations, dispatch, _Movement::default()));
		app.update();
	}

	#[test]
	fn animate_slow() {
		let mut app = setup();
		let mut config = _Config::default();
		let mut animations = _MovementAnimations::default();
		let mut dispatch = _AnimationDispatch::default();

		config
			.mock
			.expect_get_movement_data()
			.return_const((UnitsPerSecond::default(), MovementMode::Slow));
		animations
			.mock
			.expect_animation()
			.with(eq(MovementMode::Fast))
			.return_const(_Animation("fast"));
		animations
			.mock
			.expect_animation()
			.with(eq(MovementMode::Slow))
			.return_const(_Animation("slow"));

		dispatch
			.mock
			.expect_start_animation()
			.times(1)
			.with(eq(_Animation("slow")))
			.return_const(());

		app.world
			.spawn((config, animations, dispatch, _Movement::default()));
		app.update();
	}

	#[test]
	fn do_not_animate_when_no_movement_component() {
		let mut app = setup();
		let mut config = _Config::default();
		let mut animations = _MovementAnimations::default();
		let mut dispatch = _AnimationDispatch::default();

		config
			.mock
			.expect_get_movement_data()
			.return_const((UnitsPerSecond::default(), MovementMode::default()));
		animations
			.mock
			.expect_animation()
			.return_const(_Animation(""));

		dispatch
			.mock
			.expect_start_animation()
			.never()
			.return_const(());

		app.world.spawn((config, animations, dispatch));
		app.update();
	}

	#[test]
	fn remove_medium_priority_when_movement_removed() {
		let mut app = setup();
		let mut config = _Config::default();
		let mut animations = _MovementAnimations::default();
		let mut dispatch = _AnimationDispatch::default();

		config
			.mock
			.expect_get_movement_data()
			.return_const((UnitsPerSecond::default(), MovementMode::default()));
		animations
			.mock
			.expect_animation()
			.with(eq(MovementMode::Fast))
			.return_const(_Animation("fast"));
		animations
			.mock
			.expect_animation()
			.with(eq(MovementMode::Slow))
			.return_const(_Animation("slow"));

		dispatch.mock.expect_start_animation().return_const(());
		dispatch
			.mock
			.expect_stop_animation()
			.times(1)
			.return_const(());

		let agent = app
			.world
			.spawn((config, animations, dispatch, _Movement::default()))
			.id();
		app.update();

		app.world.entity_mut(agent).remove::<_Movement>();
		app.update();
	}

	#[test]
	fn animate_only_when_movement_added() {
		let mut app = setup();
		let mut config = _Config::default();
		let mut animations = _MovementAnimations::default();
		let mut dispatch = _AnimationDispatch::default();

		config
			.mock
			.expect_get_movement_data()
			.return_const((UnitsPerSecond::default(), MovementMode::default()));
		animations
			.mock
			.expect_animation()
			.return_const(_Animation("my animation"));

		dispatch
			.mock
			.expect_start_animation()
			.times(1)
			.with(eq(_Animation("my animation")))
			.return_const(());

		app.world
			.spawn((config, animations, dispatch, _Movement::default()));
		app.update();
		app.update();
	}

	#[test]
	fn animate_only_when_movement_added_and_ignore_changed() {
		let mut app = setup();
		let mut config = _Config::default();
		let mut animations = _MovementAnimations::default();
		let mut dispatch = _AnimationDispatch::default();

		config
			.mock
			.expect_get_movement_data()
			.return_const((UnitsPerSecond::default(), MovementMode::default()));
		animations
			.mock
			.expect_animation()
			.return_const(_Animation("my animation"));

		dispatch
			.mock
			.expect_start_animation()
			.times(1)
			.with(eq(_Animation("my animation")))
			.return_const(());

		let agent = app
			.world
			.spawn((config, animations, dispatch, _Movement::default()))
			.id();
		app.update();

		app.world
			.entity_mut(agent)
			.get_mut::<_Movement>()
			.unwrap()
			.0 = "CHANGED";
		app.update();
	}

	#[test]
	fn animate_again_when_config_changed() {
		let mut app = setup();
		let mut config = _Config::default();
		let mut animations = _MovementAnimations::default();
		let mut dispatch = _AnimationDispatch::default();

		config
			.mock
			.expect_get_movement_data()
			.return_const((UnitsPerSecond::default(), MovementMode::Fast));
		animations
			.mock
			.expect_animation()
			.return_const(_Animation("my animation"));

		dispatch
			.mock
			.expect_start_animation()
			.times(2)
			.with(eq(_Animation("my animation")))
			.return_const(());

		let agent = app
			.world
			.spawn((config, animations, dispatch, _Movement::default()))
			.id();
		app.update();

		app.world
			.entity_mut(agent)
			.get_mut::<_Config>()
			.unwrap()
			.mock
			.expect_get_movement_data()
			.return_const((UnitsPerSecond::default(), MovementMode::Slow));
		app.update();
	}

	#[test]
	fn no_animate_when_immobilized() {
		let mut app = setup();
		let mut config = _Config::default();
		let mut animations = _MovementAnimations::default();
		let mut dispatch = _AnimationDispatch::default();

		config
			.mock
			.expect_get_movement_data()
			.never()
			.return_const((UnitsPerSecond::default(), MovementMode::Fast));
		animations
			.mock
			.expect_animation()
			.never()
			.return_const(_Animation(""));
		dispatch
			.mock
			.expect_start_animation()
			.never()
			.return_const(());

		app.world.spawn((
			config,
			animations,
			dispatch,
			_Movement::default(),
			Immobilized,
		));
		app.update();
	}
}
