use super::effected_by_gravity::EffectedByGravity;

pub struct EffectedBy;

impl EffectedBy {
	pub fn gravity() -> EffectedByGravity {
		EffectedByGravity { pulls: vec![] }
	}
}
