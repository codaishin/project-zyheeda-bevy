pub mod deal_damage;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum EffectApplies {
	Once,
	OncePerTarget,
	Always,
}
