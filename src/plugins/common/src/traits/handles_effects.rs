use crate::effects::{force::Force, gravity::Gravity, health_damage::HealthDamage};
use bevy::prelude::{Bundle, Component};

pub trait HandlesAllEffects:
	HandlesEffect<HealthDamage> + HandlesEffect<Gravity> + HandlesEffect<Force>
{
}

impl<T> HandlesAllEffects for T where
	T: HandlesEffect<HealthDamage> + HandlesEffect<Gravity> + HandlesEffect<Force>
{
}

pub trait HandlesEffect<TEffect>
where
	TEffect: Effect,
{
	type TEffectComponent: Component;

	fn effect(effect: TEffect) -> Self::TEffectComponent;
	fn attribute(target_attribute: TEffect::TTarget) -> impl Bundle;
}

pub trait Effect {
	type TTarget;
}
