use crate::{
	components::{
		movement::{Movement, path_or_direction::PathOrDirection},
		movement_definition::MovementDefinition,
	},
	system_param::movement_param::MovementContextMut,
};
use common::{
	tools::{Units, UnitsPerSecond},
	traits::{
		animation::Animation,
		handles_movement::{MovementTarget, StartMovement},
		thread_safe::ThreadSafe,
	},
};

impl<TMotion> StartMovement for MovementContextMut<'_, TMotion>
where
	TMotion: ThreadSafe,
{
	fn start<T>(
		&mut self,
		target: T,
		radius: Units,
		speed: UnitsPerSecond,
		animation: Option<Animation>,
	) where
		T: Into<MovementTarget>,
	{
		self.entity.try_insert((
			Movement::<PathOrDirection<TMotion>>::to(target),
			MovementDefinition {
				radius,
				speed,
				animation,
			},
		));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{
			movement::{Movement, path_or_direction::PathOrDirection},
			movement_definition::MovementDefinition,
		},
		system_param::movement_param::MovementParamMut,
	};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{
		accessors::get::EntityContextMut,
		animation::{AnimationAsset, PlayMode},
		handles_movement::Movement as MovementMarker,
		thread_safe::ThreadSafe,
	};
	use test_case::test_case;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq)]
	struct _Motion;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_movement_definition() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::get_entity_context_mut(&mut p, entity, MovementMarker)
						.unwrap();
				ctx.start(
					Vec3::new(1., 2., 3.),
					Units::from(42.),
					UnitsPerSecond::from(11.),
					Some(Animation {
						asset: AnimationAsset::from("my/animation/path"),
						play_mode: PlayMode::Repeat,
					}),
				);
			})?;

		assert_eq!(
			Some(&MovementDefinition {
				radius: Units::from(42.),
				speed: UnitsPerSecond::from(11.),
				animation: Some(Animation {
					asset: AnimationAsset::from("my/animation/path"),
					play_mode: PlayMode::Repeat,
				}),
			}),
			app.world().entity(entity).get::<MovementDefinition>(),
		);
		Ok(())
	}

	#[test_case(Vec3::new(1.,2.,3.); "to point")]
	#[test_case(Dir3::NEG_X; "towards direction")]
	fn insert_movement(
		target: impl Into<MovementTarget> + Copy + ThreadSafe,
	) -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		app.world_mut()
			.run_system_once(move |mut p: MovementParamMut<_Motion>| {
				let mut ctx =
					MovementParamMut::get_entity_context_mut(&mut p, entity, MovementMarker)
						.unwrap();
				ctx.start(
					target,
					Units::from(42.),
					UnitsPerSecond::from(11.),
					Some(Animation {
						asset: AnimationAsset::from("my/animation/path"),
						play_mode: PlayMode::Repeat,
					}),
				);
			})?;

		assert_eq!(
			Some(&Movement::to(target)),
			app.world()
				.entity(entity)
				.get::<Movement<PathOrDirection<_Motion>>>(),
		);
		Ok(())
	}
}
