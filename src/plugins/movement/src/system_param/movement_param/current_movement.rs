use crate::system_param::movement_param::{MotionState, MovementContext, MovementContextMut};
use bevy::ecs::component::Component;
use common::traits::{
	accessors::get::View,
	handles_movement::{MovementTarget, SpeedToggle},
	handles_physics::CharacterMotion,
};

impl<TMotion> View<Option<MovementTarget>> for MovementContext<'_, TMotion>
where
	TMotion: Component + View<CharacterMotion>,
{
	fn view(&self) -> Option<MovementTarget> {
		let movement = match &self.motion {
			MotionState::Movement(movement) => movement,
			_ => return None,
		};

		match movement.view() {
			CharacterMotion::Direction { direction, .. } => Some(MovementTarget::Dir(direction)),
			CharacterMotion::ToTarget { target, .. } => Some(MovementTarget::Point(target)),
			CharacterMotion::Stop => None,
		}
	}
}

impl<TMotion> View<SpeedToggle> for MovementContext<'_, TMotion>
where
	TMotion: Component,
{
	fn view(&self) -> SpeedToggle {
		self.current_speed
			.as_ref()
			.map(|index| index.0)
			.unwrap_or_default()
	}
}

impl<TMotion> View<Option<MovementTarget>> for MovementContextMut<'_, TMotion>
where
	TMotion: Component + View<CharacterMotion>,
{
	fn view(&self) -> Option<MovementTarget> {
		match self.motion?.view() {
			CharacterMotion::Direction { direction, .. } => Some(MovementTarget::Dir(direction)),
			CharacterMotion::ToTarget { target, .. } => Some(MovementTarget::Point(target)),
			CharacterMotion::Stop => None,
		}
	}
}

impl<TMotion> View<SpeedToggle> for MovementContextMut<'_, TMotion>
where
	TMotion: Component,
{
	fn view(&self) -> SpeedToggle {
		self.current_speed.0
	}
}
