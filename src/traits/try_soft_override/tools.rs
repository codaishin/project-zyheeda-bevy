use super::TrySoftOverride;
use crate::{
	components::{Active, Queued, SlotKey},
	skill::Skill,
	tools::Tools,
};

impl TrySoftOverride for Tools {
	fn try_soft_override(
		running: &Skill<Active>,
		new: &Skill<Queued>,
	) -> Option<(Skill<Active>, Skill<Queued>)> {
		if !running.soft_override || !new.soft_override {
			None
		} else {
			Some(update(running, new))
		}
	}
}

fn update(running: &Skill<Active>, new: &Skill<Queued>) -> (Skill<Active>, Skill<Queued>) {
	match (running.data.slot, new.data.slot) {
		(SlotKey::Hand(running_side), SlotKey::Hand(new_side)) if running_side != new_side => {
			(new.marker.skill_modify.update_dual_fn)(running, new)
		}
		_ => (new.marker.skill_modify.update_single_fn)(running, new),
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
		fn modify_single(
			_running: &Skill<Active>,
			_next: &Skill<Queued>,
		) -> (Skill<Active>, Skill<Queued>) {
			default()
		}

		#[allow(dead_code)]
		fn modify_dual(
			_running: &Skill<Active>,
			_next: &Skill<Queued>,
		) -> (Skill<Active>, Skill<Queued>) {
			default()
		}
	}

	#[test]
	fn call_modify_dual_when_different_sides() {
		let running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Right),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					update_dual_fn: |_, _| default(),
					update_single_fn: |_, _| panic!("single should not be called"),
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&running, &new);
	}

	#[test]
	fn call_modify_single_when_same_sides() {
		let running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					update_dual_fn: |_, _| panic!("dual should not be called"),
					update_single_fn: |_, _| default(),
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&running, &new);
	}

	#[test]
	fn call_modify_single_when_running_no_hands() {
		let running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Legs,
				..default()
			},
			..default()
		};
		let new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					update_dual_fn: |_, _| panic!("dual should not be called"),
					update_single_fn: |_, _| default(),
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&running, &new);
	}

	#[test]
	fn call_modify_single_when_new_no_hands() {
		let running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Legs,
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					update_dual_fn: |_, _| panic!("dual should not be called"),
					update_single_fn: |_, _| default(),
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&running, &new);
	}

	#[test]
	fn call_modify_dual_args() {
		let running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Right),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					update_dual_fn: Mock_Fns::modify_dual,
					update_single_fn: |_, _| default(),
				},
				..default()
			},
			..default()
		};
		let ctx = Mock_Fns::modify_dual_context();
		ctx.expect()
			.times(1)
			.with(eq(running), eq(new))
			.return_const((
				Skill {
					name: "r_u",
					..default()
				},
				Skill {
					name: "n_u",
					..default()
				},
			));

		Tools::try_soft_override(&running, &new);
	}

	#[test]
	fn call_modify_single_args() {
		let running = Skill::<Active> {
			soft_override: true,
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let new = Skill::<Queued> {
			soft_override: true,
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: MarkerMeta {
				skill_modify: SkillModify {
					update_dual_fn: |_, _| default(),
					update_single_fn: Mock_Fns::modify_single,
				},
				..default()
			},
			..default()
		};
		let ctx = Mock_Fns::modify_single_context();
		ctx.expect()
			.times(1)
			.with(eq(running), eq(new))
			.return_const((
				Skill {
					name: "r_u",
					..default()
				},
				Skill {
					name: "n_u",
					..default()
				},
			));

		Tools::try_soft_override(&running, &new);
	}

	#[test]
	fn no_modify_call_when_running_sof_override_false() {
		let running = Skill::<Active> {
			soft_override: false,
			..default()
		};
		let new = Skill::<Queued> {
			soft_override: true,
			marker: MarkerMeta {
				skill_modify: SkillModify {
					update_dual_fn: |_, _| panic!("dual should not be called"),
					update_single_fn: |_, _| panic!("single should not be called"),
				},
				..default()
			},
			..default()
		};

		Tools::try_soft_override(&running, &new);
	}
}
