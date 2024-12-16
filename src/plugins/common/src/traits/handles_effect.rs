use crate::effects::{deal_damage::DealDamage, force_shield::ForceShield, gravity::Gravity};
use bevy::prelude::{Bundle, Component};

pub trait HandlesAllEffects:
	HandlesEffect<DealDamage> + HandlesEffect<Gravity> + HandlesEffect<ForceShield>
{
}

impl<T> HandlesAllEffects for T where
	T: HandlesEffect<DealDamage> + HandlesEffect<Gravity> + HandlesEffect<ForceShield>
{
}

pub trait HandlesEffect<TEffect> {
	type TTarget;
	type TEffectComponent: Component;

	fn effect(effect: TEffect) -> Self::TEffectComponent;
	fn attribute(target_attribute: Self::TTarget) -> impl Bundle;
}
