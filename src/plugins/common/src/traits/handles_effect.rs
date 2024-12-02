use crate::effects::{deal_damage::DealDamage, gravity::Gravity};
use bevy::prelude::Bundle;

pub trait HandlesAllEffects: HandlesEffect<DealDamage> + HandlesEffect<Gravity> {}

impl<T> HandlesAllEffects for T where T: HandlesEffect<DealDamage> + HandlesEffect<Gravity> {}

pub trait HandlesEffect<TEffect> {
	type TTarget;

	fn effect(effect: TEffect) -> impl Bundle;
	fn attribute(target_attribute: Self::TTarget) -> impl Bundle;
}
