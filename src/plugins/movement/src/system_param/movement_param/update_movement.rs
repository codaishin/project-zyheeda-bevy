use crate::system_param::movement_param::MovementContextMut;
use bevy::ecs::component::Component;
use common::{tools::UnitsPerSecond, traits::handles_movement::UpdateMovement};

impl<TMotion> UpdateMovement for MovementContextMut<'_, TMotion>
where
	TMotion: Component,
{
	fn update(&mut self, _: UnitsPerSecond) {}
}
