use super::TrySoftOverride;
use crate::{
	components::{Active, Queued, Skill, SlotKey},
	tools::Tools,
};

impl TrySoftOverride for Tools {
	fn try_soft_override(running: &mut Skill<Active>, new: &mut Skill<Queued>) -> bool {
		if !running.soft_override || !new.soft_override {
			return false;
		}

		let modify_fn = get_skill_modify_fn(running, new);
		modify_fn(running, new);
		true
	}
}

fn get_skill_modify_fn(
	running: &mut Skill<Active>,
	new: &mut Skill<Queued>,
) -> fn(&mut Skill<Active>, &mut Skill<Queued>) {
	match (running.data.slot, new.data.slot) {
		(SlotKey::Hand(running_side), SlotKey::Hand(new_side)) if running_side != new_side => {
			new.marker.skill_modify.modify_dual_fn
		}
		_ => new.marker.skill_modify.modify_single_fn,
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Queued, Side, SlotKey},
		markers::meta::{MarkerMeta, SkillModify},
	};
	use bevy::utils::default;
	use mockall::{automock, predicate::eq};

	struct _Fns;

	#[automock]
	impl _Fns {
		#[allow(dead_code)]
		fn modify_single(_running: &mut Skill<Active>, _next: &mut Skill<Queued>) {}

		#[allow(dead_code)]
		fn modify_dual(_running: &mut Skill<Active>, _next: &mut Skill<Queued>) {}
	}

	#[test]
	fn call_modify_dual_when_different_sides() {
		let mut running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Right),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					modify_dual_fn: |_, _| {},
					modify_single_fn: |_, _| panic!("single should not be called"),
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&mut running, &mut new);
	}

	#[test]
	fn call_modify_single_when_same_sides() {
		let mut running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					modify_dual_fn: |_, _| panic!("dual should not be called"),
					modify_single_fn: |_, _| {},
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&mut running, &mut new);
	}

	#[test]
	fn call_modify_single_when_running_no_hands() {
		let mut running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Legs,
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					modify_dual_fn: |_, _| panic!("dual should not be called"),
					modify_single_fn: |_, _| {},
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&mut running, &mut new);
	}

	#[test]
	fn call_modify_single_when_new_no_hands() {
		let mut running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Legs,
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					modify_dual_fn: |_, _| panic!("dual should not be called"),
					modify_single_fn: |_, _| {},
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&mut running, &mut new);
	}

	#[test]
	fn call_modify_dual_args() {
		let mut running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Right),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					modify_dual_fn: Mock_Fns::modify_dual,
					modify_single_fn: |_, _| {},
				},
				..default()
			},
			..default()
		};
		let ctx = Mock_Fns::modify_dual_context();
		ctx.expect()
			.times(1)
			.with(eq(running), eq(new))
			.return_const(());

		Tools::try_soft_override(&mut running, &mut new);
	}

	#[test]
	fn call_modify_single_args() {
		let mut running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					modify_dual_fn: |_, _| {},
					modify_single_fn: Mock_Fns::modify_single,
				},
				..default()
			},
			..default()
		};
		let ctx = Mock_Fns::modify_single_context();
		ctx.expect()
			.times(1)
			.with(eq(running), eq(new))
			.return_const(());

		Tools::try_soft_override(&mut running, &mut new);
	}

	#[test]
	fn no_modify_call_when_running_sof_override_false() {
		let mut running = Skill::<Active> {
			soft_override: false,
			..default()
		};
		let mut new = Skill::<Queued> {
			soft_override: true,
			marker: MarkerMeta {
				skill_modify: SkillModify {
					modify_dual_fn: |_, _| panic!("dual should not be called"),
					modify_single_fn: |_, _| panic!("single should not be called"),
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&mut running, &mut new);
	}
}
