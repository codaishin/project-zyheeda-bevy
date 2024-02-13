pub(crate) mod damage_health;

pub(crate) trait ActOn<TTarget> {
	fn act_on(&mut self, target: &mut TTarget);
}
