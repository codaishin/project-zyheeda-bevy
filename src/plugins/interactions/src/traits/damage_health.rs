use super::ActOn;
use crate::components::DealsDamage;
use common::components::Health;

impl ActOn<Health> for DealsDamage {
	fn act_on(&mut self, health: &mut Health) {
		health.current -= self.0;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn deal_damage() {
		let mut damage = DealsDamage(42);
		let mut health = Health::new(100);
		damage.act_on(&mut health);

		assert_eq!(
			Health {
				current: 58,
				max: 100
			},
			health
		);
	}
}
