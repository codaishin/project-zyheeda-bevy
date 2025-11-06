use crate::components::movement_definition::MovementDefinition;
use bevy::{ecs::component::Mutable, prelude::*};
use common::{
	components::immobilized::Immobilized,
	traits::animation::{AnimationPriority, SetAnimations, StopAnimation},
};

impl MovementDefinition {
	#[allow(clippy::type_complexity)]
	pub(crate) fn animate_movement<TMovement, TAnimationDispatch>(
		mut agents: Query<
			(Ref<Self>, &mut TAnimationDispatch, Ref<TMovement>),
			Without<Immobilized>,
		>,
		mut agents_without_movement: Query<&mut TAnimationDispatch, Without<TMovement>>,
		mut removed_movements: RemovedComponents<TMovement>,
	) where
		TMovement: Component,
		TAnimationDispatch: Component<Mutability = Mutable> + SetAnimations + StopAnimation,
	{
		for (definition, dispatch, movement) in &mut agents {
			Self::start_animation(definition, dispatch, movement);
		}

		for entity in removed_movements.read() {
			Self::stop_animation(entity, &mut agents_without_movement);
		}
	}

	fn start_animation<TMovement, TAnimationDispatch>(
		config: Ref<Self>,
		mut dispatch: Mut<TAnimationDispatch>,
		movement: Ref<TMovement>,
	) where
		TAnimationDispatch: SetAnimations,
	{
		if !movement.is_added() && !config.is_changed() {
			return;
		}
		let Some(animation) = &config.animation else {
			return;
		};
		dispatch.set_animations(Move, [animation.clone()]);
	}

	fn stop_animation<TMovement, TAnimationDispatch>(
		entity: Entity,
		agent_without_movement: &mut Query<&mut TAnimationDispatch, Without<TMovement>>,
	) where
		TMovement: Component,
		TAnimationDispatch: Component<Mutability = Mutable> + StopAnimation,
	{
		let Ok(mut dispatch) = agent_without_movement.get_mut(entity) else {
			return;
		};
		dispatch.stop_animation(Move);
	}
}

#[derive(Debug, PartialEq)]
struct Move;

impl From<Move> for AnimationPriority {
	fn from(_: Move) -> Self {
		AnimationPriority::Medium
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::animation::{Animation, AnimationPath, PlayMode};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::ops::DerefMut;
	use testing::{NestedMocks, SingleThreadedApp};

	#[derive(Component)]
	struct _Movement;

	#[derive(Component, NestedMocks)]
	struct _AnimationDispatch {
		mock: Mock_AnimationDispatch,
	}

	impl SetAnimations for _AnimationDispatch {
		fn set_animations<TLayer, TAnimations>(&mut self, layer: TLayer, animations: TAnimations)
		where
			TLayer: Into<AnimationPriority> + 'static,
			TAnimations: IntoIterator<Item = Animation> + 'static,
		{
			self.mock.set_animations(layer, animations)
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
		impl SetAnimations for _AnimationDispatch {
			fn set_animations<TLayer, TAnimations>(&mut self, layer: TLayer, animations: TAnimations)
			where
				TLayer: Into<AnimationPriority> + 'static,
				TAnimations: IntoIterator<Item = Animation> + 'static;
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
			MovementDefinition::animate_movement::<_Movement, _AnimationDispatch>,
		);

		app
	}

	#[test]
	fn animate() {
		let mut app = setup();
		app.world_mut().spawn((
			MovementDefinition {
				animation: Some(Animation::new(
					AnimationPath::from("fast"),
					PlayMode::Repeat,
				)),
				..default()
			},
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_set_animations::<Move, [Animation; 1]>()
					.times(1)
					.with(
						eq(Move),
						eq([Animation::new(
							AnimationPath::from("fast"),
							PlayMode::Repeat,
						)]),
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
			MovementDefinition {
				animation: Some(Animation::new(AnimationPath::from(""), PlayMode::Repeat)),
				..default()
			},
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_set_animations::<Move, [Animation; 1]>()
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
				MovementDefinition {
					animation: Some(Animation::new(AnimationPath::from(""), PlayMode::Repeat)),
					..default()
				},
				_AnimationDispatch::new().with_mock(|mock| {
					mock.expect_set_animations::<Move, [Animation; 1]>()
						.return_const(());
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
			MovementDefinition {
				animation: Some(Animation::new(AnimationPath::from(""), PlayMode::Repeat)),
				..default()
			},
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_set_animations::<Move, [Animation; 1]>()
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
				MovementDefinition {
					animation: Some(Animation::new(AnimationPath::from(""), PlayMode::Repeat)),
					..default()
				},
				_AnimationDispatch::new().with_mock(|mock| {
					mock.expect_set_animations::<Move, [Animation; 1]>()
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
				MovementDefinition {
					animation: Some(Animation::new(AnimationPath::from(""), PlayMode::Repeat)),
					..default()
				},
				_AnimationDispatch::new().with_mock(|mock| {
					mock.expect_set_animations::<Move, [Animation; 1]>()
						.times(2)
						.return_const(());
				}),
				_Movement,
			))
			.id();
		app.update();

		app.world_mut()
			.entity_mut(agent)
			.get_mut::<MovementDefinition>()
			.unwrap()
			.deref_mut();

		app.update();
	}

	#[test]
	fn no_animate_when_immobilized() {
		let mut app = setup();
		app.world_mut().spawn((
			MovementDefinition {
				animation: Some(Animation::new(AnimationPath::from(""), PlayMode::Repeat)),
				..default()
			},
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_set_animations::<Move, [Animation; 1]>()
					.never()
					.return_const(());
			}),
			_Movement,
			Immobilized,
		));
		app.update();
	}
}
