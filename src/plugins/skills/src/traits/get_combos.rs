use super::{Combo, GetCombos};
use crate::{items::slot_key::SlotKey, skills::Skill};
use common::traits::{get::Get, iteration::IterFinite};

impl<T: Get<Vec<SlotKey>, Skill>> GetCombos for T {
	fn combos(&self) -> Vec<Combo> {
		combos(self, vec![])
	}
}

fn combos(combo_source: &impl Get<Vec<SlotKey>, Skill>, slot_keys: Vec<SlotKey>) -> Vec<Combo> {
	SlotKey::iterator()
		.filter_map(get_combos_step(combo_source, slot_keys))
		.flat_map(append_followup_combo_steps(combo_source))
		.collect()
}

fn get_combos_step<'a>(
	combo_source: &'a impl Get<Vec<SlotKey>, Skill>,
	slot_keys: Vec<SlotKey>,
) -> impl FnMut(SlotKey) -> Option<((SlotKey, &'a Skill), Vec<SlotKey>)> {
	move |slot_key| {
		let slot_keys = push_cloned(&slot_keys, slot_key);
		let skill = combo_source.get(&slot_keys)?;
		Some(((slot_key, skill), slot_keys))
	}
}

fn append_followup_combo_steps<'a>(
	combo_source: &'a impl Get<Vec<SlotKey>, Skill>,
) -> impl FnMut(((SlotKey, &'a Skill), Vec<SlotKey>)) -> Vec<Combo<'a>> + 'a {
	|(combo_step, slot_keys)| {
		let followups = combos(combo_source, slot_keys);
		complete_combos(combo_step, followups)
	}
}

fn push_cloned(src: &[SlotKey], value: SlotKey) -> Vec<SlotKey> {
	let mut src = src.to_vec();
	src.push(value);
	src
}

fn complete_combos<'a>(
	combo_step: (SlotKey, &'a Skill),
	possible_followup: Vec<Combo<'a>>,
) -> Vec<Combo<'a>> {
	let root_steps = vec![combo_step];

	if possible_followup.is_empty() {
		return vec![root_steps];
	}

	possible_followup
		.into_iter()
		.map(|remaining_steps| root_steps.iter().cloned().chain(remaining_steps).collect())
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::combos::ComboNode;
	use bevy::utils::default;
	use common::components::Side;

	#[test]
	fn get_single_single_combo_with_single_skill() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::default(),
			),
		)]);

		assert_eq!(
			vec![vec![(
				SlotKey::Hand(Side::Main),
				&Skill {
					name: "some skill".to_owned(),
					..default()
				}
			)]],
			combos.combos()
		)
	}

	#[test]
	fn get_multiple_combos_with_single_skill() {
		let combos = ComboNode::new([
			(
				SlotKey::Hand(Side::Main),
				(
					Skill {
						name: "some right skill".to_owned(),
						..default()
					},
					ComboNode::default(),
				),
			),
			(
				SlotKey::Hand(Side::Off),
				(
					Skill {
						name: "some left skill".to_owned(),
						..default()
					},
					ComboNode::default(),
				),
			),
		]);

		assert_eq!(
			vec![
				vec![(
					SlotKey::Hand(Side::Main),
					&Skill {
						name: "some right skill".to_owned(),
						..default()
					}
				)],
				vec![(
					SlotKey::Hand(Side::Off),
					&Skill {
						name: "some left skill".to_owned(),
						..default()
					}
				)]
			],
			combos.combos()
		)
	}

	#[test]
	fn get_single_combo_with_multiple_skills() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Off),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::default(),
					),
				)]),
			),
		)]);

		assert_eq!(
			vec![vec![
				(
					SlotKey::Hand(Side::Main),
					&Skill {
						name: "some skill".to_owned(),
						..default()
					}
				),
				(
					SlotKey::Hand(Side::Off),
					&Skill {
						name: "some child skill".to_owned(),
						..default()
					}
				)
			]],
			combos.combos()
		)
	}

	#[test]
	fn get_multiple_combos_with_multiple_child_skills() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([
					(
						SlotKey::Hand(Side::Main),
						(
							Skill {
								name: "some right child skill".to_owned(),
								..default()
							},
							ComboNode::default(),
						),
					),
					(
						SlotKey::Hand(Side::Off),
						(
							Skill {
								name: "some left child skill".to_owned(),
								..default()
							},
							ComboNode::default(),
						),
					),
				]),
			),
		)]);

		assert_eq!(
			vec![
				vec![
					(
						SlotKey::Hand(Side::Main),
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						SlotKey::Hand(Side::Main),
						&Skill {
							name: "some right child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						SlotKey::Hand(Side::Main),
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						SlotKey::Hand(Side::Off),
						&Skill {
							name: "some left child skill".to_owned(),
							..default()
						}
					)
				]
			],
			combos.combos()
		)
	}

	#[test]
	fn get_multiple_combo_with_multiple_deep_child_skills() {
		let combos = ComboNode::new([(
			SlotKey::Hand(Side::Main),
			(
				Skill {
					name: "some skill".to_owned(),
					..default()
				},
				ComboNode::new([(
					SlotKey::Hand(Side::Main),
					(
						Skill {
							name: "some child skill".to_owned(),
							..default()
						},
						ComboNode::new([
							(
								SlotKey::Hand(Side::Main),
								(
									Skill {
										name: "some right child skill".to_owned(),
										..default()
									},
									ComboNode::default(),
								),
							),
							(
								SlotKey::Hand(Side::Off),
								(
									Skill {
										name: "some left child skill".to_owned(),
										..default()
									},
									ComboNode::default(),
								),
							),
						]),
					),
				)]),
			),
		)]);

		assert_eq!(
			vec![
				vec![
					(
						SlotKey::Hand(Side::Main),
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						SlotKey::Hand(Side::Main),
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						SlotKey::Hand(Side::Main),
						&Skill {
							name: "some right child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						SlotKey::Hand(Side::Main),
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						SlotKey::Hand(Side::Main),
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						SlotKey::Hand(Side::Off),
						&Skill {
							name: "some left child skill".to_owned(),
							..default()
						}
					)
				]
			],
			combos.combos()
		)
	}
}
