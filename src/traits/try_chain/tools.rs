use super::TryChain;
use crate::{
	components::{Active, Queued, Skill, SlotKey},
	tools::Tools,
};

fn get_modify(
	running: &mut Skill<Active>,
	new: &mut Skill<Queued>,
) -> fn(&mut Skill<Active>, &mut Skill<Queued>) {
	match (running.data.slot, new.data.slot) {
		(SlotKey::Hand(running_side), SlotKey::Hand(new_side)) if running_side != new_side => {
			new.marker.chain.modify_dual
		}
		_ => new.marker.chain.modify_single,
	}
}

impl TryChain for Tools {
	fn try_chain(running: &mut Skill<Active>, new: &mut Skill<Queued>) -> bool {
		let can_chain = new.marker.chain.can_chain;
		if !can_chain(running, new) {
			return false;
		}

		let modify = get_modify(running, new);
		modify(running, new);
		true
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{Queued, Side, SlotKey},
		markers::meta::{Chain, MarkerMeta},
	};
	use bevy::utils::default;
	use mockall::{automock, predicate::eq};

	struct _Fns;

	#[automock]
	impl _Fns {
		#[allow(dead_code)]
		fn can_chain(_running: &Skill<Active>, _next: &Skill<Queued>) -> bool {
			false
		}

		#[allow(dead_code)]
		fn modify_single(_running: &mut Skill<Active>, _next: &mut Skill<Queued>) {}

		#[allow(dead_code)]
		fn modify_dual(_running: &mut Skill<Active>, _next: &mut Skill<Queued>) {}
	}

	#[test]
	fn call_chain_can_chain() {
		let mut running = Skill::<Active> { ..default() };
		let mut new = Skill::<Queued> {
			marker: MarkerMeta {
				chain: Chain {
					can_chain: Mock_Fns::can_chain,
					..default()
				},
				..default()
			},
			..default()
		};
		let ctx = Mock_Fns::can_chain_context();

		ctx.expect()
			.times(1)
			.with(eq(running), eq(new))
			.return_const(true);

		_ = Tools::try_chain(&mut running, &mut new);
	}

	#[test]
	fn return_call_chain_result() {
		static mut LAST_RESULT: bool = false;

		fn can_chain(_running: &Skill<Active>, _next: &Skill<Queued>) -> bool {
			unsafe {
				LAST_RESULT = !LAST_RESULT;
				LAST_RESULT
			}
		}

		let mut running = Skill::<Active> { ..default() };
		let mut new = Skill::<Queued> {
			marker: MarkerMeta {
				chain: Chain {
					can_chain,
					..default()
				},
				..default()
			},
			..default()
		};

		assert_eq!(
			[true, false, true],
			[
				Tools::try_chain(&mut running, &mut new),
				Tools::try_chain(&mut running, &mut new),
				Tools::try_chain(&mut running, &mut new),
			]
		)
	}

	#[test]
	fn call_modify_dual_when_different_sides() {
		let mut running = Skill::<Active> {
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			data: Queued {
				slot: SlotKey::Hand(Side::Right),
				..default()
			},
			marker: MarkerMeta {
				chain: Chain {
					can_chain: |_, _| true,
					modify_dual: |_, _| {},
					modify_single: |_, _| panic!("single should not be called"),
				},
				..default()
			},
			..default()
		};

		Tools::try_chain(&mut running, &mut new);
	}

	#[test]
	fn call_modify_single_when_same_sides() {
		let mut running = Skill::<Active> {
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: MarkerMeta {
				chain: Chain {
					can_chain: |_, _| true,
					modify_dual: |_, _| panic!("dual should not be called"),
					modify_single: |_, _| {},
				},
				..default()
			},
			..default()
		};

		Tools::try_chain(&mut running, &mut new);
	}

	#[test]
	fn call_modify_single_when_running_no_hands() {
		let mut running = Skill::<Active> {
			data: Active {
				slot: SlotKey::Legs,
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: MarkerMeta {
				chain: Chain {
					can_chain: |_, _| true,
					modify_dual: |_, _| panic!("dual should not be called"),
					modify_single: |_, _| {},
				},
				..default()
			},
			..default()
		};

		Tools::try_chain(&mut running, &mut new);
	}

	#[test]
	fn call_modify_single_when_new_no_hands() {
		let mut running = Skill::<Active> {
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			data: Queued {
				slot: SlotKey::Legs,
				..default()
			},
			marker: MarkerMeta {
				chain: Chain {
					can_chain: |_, _| true,
					modify_dual: |_, _| panic!("dual should not be called"),
					modify_single: |_, _| {},
				},
				..default()
			},
			..default()
		};

		Tools::try_chain(&mut running, &mut new);
	}

	#[test]
	fn call_modify_dual_args() {
		let mut running = Skill::<Active> {
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			data: Queued {
				slot: SlotKey::Hand(Side::Right),
				..default()
			},
			marker: MarkerMeta {
				chain: Chain {
					can_chain: |_, _| true,
					modify_dual: Mock_Fns::modify_dual,
					modify_single: |_, _| {},
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

		Tools::try_chain(&mut running, &mut new);
	}

	#[test]
	fn call_modify_single_args() {
		let mut running = Skill::<Active> {
			data: Active {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			..default()
		};
		let mut new = Skill::<Queued> {
			data: Queued {
				slot: SlotKey::Hand(Side::Left),
				..default()
			},
			marker: MarkerMeta {
				chain: Chain {
					can_chain: |_, _| true,
					modify_dual: |_, _| {},
					modify_single: Mock_Fns::modify_single,
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

		Tools::try_chain(&mut running, &mut new);
	}

	#[test]
	fn no_modify_call_when_can_chain_false() {
		let mut running = Skill::<Active> { ..default() };
		let mut new = Skill::<Queued> {
			marker: MarkerMeta {
				chain: Chain {
					can_chain: |_, _| false,
					modify_dual: |_, _| panic!("dual should not be called"),
					modify_single: |_, _| panic!("single should not be called"),
				},
				..default()
			},
			..default()
		};

		Tools::try_chain(&mut running, &mut new);
	}
}
