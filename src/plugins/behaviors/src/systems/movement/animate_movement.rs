use bevy::prelude::*;
use common::{
	components::Immobilized,
	traits::{
		accessors::get::GetterRef,
		animation::{Animation, AnimationPriority, StartAnimation, StopAnimation},
	},
};

#[derive(Debug, PartialEq)]
struct Move;

impl From<Move> for AnimationPriority {
	fn from(_: Move) -> Self {
		AnimationPriority::Medium
	}
}

#[allow(clippy::type_complexity)]
pub(crate) fn animate_movement<TAgent, TMovement, TAnimationDispatch>(
	mut agents: Query<(Ref<TAgent>, Ref<TMovement>, &mut TAnimationDispatch), Without<Immobilized>>,
	mut agents_without_movement: Query<&mut TAnimationDispatch, Without<TMovement>>,
	mut removed_movements: RemovedComponents<TMovement>,
) where
	TAgent: Component + GetterRef<Animation>,
	TMovement: Component,
	TAnimationDispatch: Component + StartAnimation + StopAnimation,
{
	for (agent, movement, dispatch) in &mut agents {
		start_animation(agent, movement, dispatch);
	}

	for entity in removed_movements.read() {
		stop_animation(entity, &mut agents_without_movement);
	}
}

fn start_animation<TAgent, TMovement, TAnimationDispatch: StartAnimation>(
	agent: Ref<TAgent>,
	movement: Ref<TMovement>,
	mut dispatch: Mut<TAnimationDispatch>,
) where
	TAgent: GetterRef<Animation>,
{
	if !movement.is_added() && !agent.is_changed() {
		return;
	}
	let animation = agent.get();
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
	use common::{
		test_tools::utils::SingleThreadedApp,
		traits::{
			animation::{Animation, PlayMode},
			load_asset::Path,
			nested_mock::NestedMocks,
		},
	};
	use macros::NestedMocks;
	use mockall::{mock, predicate::eq};
	use std::ops::DerefMut;

	#[derive(Component)]
	struct _Agent(Animation);

	impl Default for _Agent {
		fn default() -> Self {
			_Agent(Animation::new(Path::from("not set"), PlayMode::Replay))
		}
	}

	impl GetterRef<Animation> for _Agent {
		fn get(&self) -> &Animation {
			let _Agent(animation) = self;
			animation
		}
	}

	#[derive(Component)]
	struct _Movement;

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
			animate_movement::<_Agent, _Movement, _AnimationDispatch>,
		);

		app
	}

	#[test]
	fn animate_fast() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent(Animation::new(Path::from("fast"), PlayMode::Repeat)),
			_Movement,
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation()
					.times(1)
					.with(
						eq(Move),
						eq(Animation::new(Path::from("fast"), PlayMode::Repeat)),
					)
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn animate_slow() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent(Animation::new(Path::from("slow"), PlayMode::Repeat)),
			_Movement,
			_AnimationDispatch::new().with_mock(|mock| {
				mock.expect_start_animation()
					.times(1)
					.with(
						eq(Move),
						eq(Animation::new(Path::from("slow"), PlayMode::Repeat)),
					)
					.return_const(());
			}),
		));

		app.update();
	}

	#[test]
	fn do_not_animate_when_no_movement_component() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent::default(),
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
				_Agent::default(),
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
			_Agent::default(),
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
				_Agent::default(),
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
				_Agent::default(),
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
			.get_mut::<_Agent>()
			.unwrap()
			.deref_mut();

		app.update();
	}

	#[test]
	fn no_animate_when_immobilized() {
		let mut app = setup();
		app.world_mut().spawn((
			_Agent::default(),
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
