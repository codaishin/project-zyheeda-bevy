use crate::system_param::movement_param::{MovementContext, MovementContextMut};
use bevy::ecs::component::Component;
use common::traits::{
	accessors::get::GetProperty,
	handles_movement::{CurrentMovement, MovementTarget},
	handles_physics::CharacterMotion,
};

impl<TMotion> CurrentMovement for MovementContext<'_, TMotion>
where
	TMotion: Component + GetProperty<CharacterMotion>,
{
	fn current_movement(&self) -> Option<MovementTarget> {
		let movement = match self {
			MovementContext::Movement(movement) => movement,
			_ => return None,
		};

		match movement.get_property() {
			CharacterMotion::Direction { direction, .. } => Some(MovementTarget::Dir(direction)),
			CharacterMotion::ToTarget { target, .. } => Some(MovementTarget::Point(target)),
			CharacterMotion::Stop => None,
		}
	}
}

impl<TMotion> CurrentMovement for MovementContextMut<'_, TMotion>
where
	TMotion: Component + GetProperty<CharacterMotion>,
{
	fn current_movement(&self) -> Option<MovementTarget> {
		match self.motion?.get_property() {
			CharacterMotion::Direction { direction, .. } => Some(MovementTarget::Dir(direction)),
			CharacterMotion::ToTarget { target, .. } => Some(MovementTarget::Point(target)),
			CharacterMotion::Stop => None,
		}
	}
}
