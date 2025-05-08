use crate::traits::change_per_frame::MinDistance;
use bevy::{ecs::event::Event, math::Vec3};
use common::tools::speed::Speed;
use std::{marker::PhantomData, time::Duration};

#[derive(Event, Debug, PartialEq, Clone, Copy)]
pub(crate) struct MovePointerEvent(pub(crate) Vec3);

impl From<Vec3> for MovePointerEvent {
	fn from(translation: Vec3) -> Self {
		Self(translation)
	}
}

#[derive(Event, Debug, PartialEq, Clone, Copy)]
pub(crate) struct MoveDirectionalEvent<TMethod> {
	pub(crate) target: Vec3,
	_m: PhantomData<TMethod>,
}

impl<TMethod> From<Vec3> for MoveDirectionalEvent<TMethod> {
	fn from(target: Vec3) -> Self {
		Self {
			target,
			_m: PhantomData,
		}
	}
}

impl<TMethod> MinDistance for MoveDirectionalEvent<TMethod>
where
	TMethod: MinDistance,
{
	fn min_distance(speed: Speed, delta: Duration) -> f32 {
		TMethod::min_distance(speed, delta)
	}
}
