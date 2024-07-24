use super::{Combo, GetCombos};
use crate::{items::slot_key::SlotKey, skills::Skill};
use common::traits::{get::Get, iteration::IterFinite};

impl<T: Get<Vec<SlotKey>, Skill>> GetCombos for T {
	fn combos(&self) -> Vec<Combo> {
		combos(self, vec![])
	}
}

fn combos(combo_source: &impl Get<Vec<SlotKey>, Skill>, key_path: Vec<SlotKey>) -> Vec<Combo> {
	SlotKey::iterator()
		.filter_map(get_combo_step(combo_source, key_path))
		.flat_map(append_followup_combo_steps(combo_source))
		.collect()
}

fn get_combo_step<'a>(
	combo_source: &'a impl Get<Vec<SlotKey>, Skill>,
	key_path: Vec<SlotKey>,
) -> impl FnMut(SlotKey) -> Option<(Vec<SlotKey>, &'a Skill)> {
	move |slot_key| {
		let key_path = append_key(key_path.clone(), slot_key);
		let skill = combo_source.get(&key_path)?;
		Some((key_path, skill))
	}
}

fn append_followup_combo_steps<'a>(
	combo_source: &'a impl Get<Vec<SlotKey>, Skill>,
) -> impl FnMut((Vec<SlotKey>, &'a Skill)) -> Vec<Combo<'a>> + 'a {
	|combo_step| {
		let combo_step_key_path = combo_step.0.clone();
		let followup_combo_steps = combos(combo_source, combo_step_key_path);
		append_followups(combo_step, followup_combo_steps)
	}
}

fn append_key(mut key_path: Vec<SlotKey>, key: SlotKey) -> Vec<SlotKey> {
	key_path.push(key);
	key_path
}

fn append_followups<'a>(
	combo_step: (Vec<SlotKey>, &'a Skill),
	followups: Vec<Combo<'a>>,
) -> Vec<Combo<'a>> {
	let combo_steps = vec![combo_step];

	if followups.is_empty() {
		return vec![combo_steps];
	}

	followups
		.into_iter()
		.map(|followup_steps| combo_steps.iter().cloned().chain(followup_steps).collect())
		.collect()
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::combo_node::ComboNode;
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
				vec![SlotKey::Hand(Side::Main)],
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
					vec![SlotKey::Hand(Side::Main)],
					&Skill {
						name: "some right skill".to_owned(),
						..default()
					}
				)],
				vec![(
					vec![SlotKey::Hand(Side::Off)],
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
					vec![SlotKey::Hand(Side::Main)],
					&Skill {
						name: "some skill".to_owned(),
						..default()
					}
				),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
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
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some right child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
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
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
						],
						&Skill {
							name: "some right child skill".to_owned(),
							..default()
						}
					)
				],
				vec![
					(
						vec![SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some skill".to_owned(),
							..default()
						}
					),
					(
						vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						&Skill {
							name: "some child skill".to_owned(),
							..default()
						}
					),
					(
						vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Off),
						],
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
