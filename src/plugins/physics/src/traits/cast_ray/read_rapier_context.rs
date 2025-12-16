use super::{CastRayContinuously, GetContinuousSortedRayCaster};
use bevy::ecs::error::BevyError;
use bevy_rapier3d::plugin::{RapierContext, ReadRapierContext};

impl<T> GetContinuousSortedRayCaster<T> for ReadRapierContext<'_, '_>
where
	for<'a> RapierContext<'a>: CastRayContinuously<T>,
{
	type TError = BevyError;
	type TRayCaster<'a>
		= RapierContext<'a>
	where
		Self: 'a;

	fn get_continuous_sorted_ray_caster(&self) -> Result<Self::TRayCaster<'_>, Self::TError> {
		self.single()
	}
}
