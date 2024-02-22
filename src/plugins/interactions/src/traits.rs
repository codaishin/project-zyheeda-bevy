pub(crate) mod damage_health;
pub(crate) mod rapier_context;

pub trait ActOn<TTarget> {
	fn act_on(&mut self, target: &mut TTarget);
}
