use crate::components::combo_overview::ComboSkill;
use bevy::prelude::*;
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::GetProperty,
		handles_loadout::{
			combos::GetCombosOrdered,
			skills::{GetSkillId, SkillIcon, SkillToken},
		},
	},
};
use std::{collections::HashSet, fmt::Debug};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Symbol {
	Root,
	Line,
	Corner,
	Empty,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ComboTreeElement<TKey, TSkill> {
	Node { key_path: Vec<TKey>, skill: TSkill },
	Leaf { key_path: Vec<TKey>, skill: TSkill },
	Symbol(Symbol),
}

pub(crate) type ComboTreeLayout<TKey, TSkill> = Vec<Vec<ComboTreeElement<TKey, TSkill>>>;

pub(crate) trait BuildComboTreeLayout<TId>
where
	TId: Debug + PartialEq + Clone,
{
	fn build_combo_tree_layout(&self) -> ComboTreeLayout<SlotKey, ComboSkill<TId>>;
}

impl<T, TId> BuildComboTreeLayout<TId> for T
where
	TId: Debug + PartialEq + Clone,
	T: GetCombosOrdered<TSkill: GetProperty<SkillToken> + GetProperty<SkillIcon> + GetSkillId<TId>>,
{
	fn build_combo_tree_layout(&self) -> ComboTreeLayout<SlotKey, ComboSkill<TId>> {
		let mut get_first_symbol = get_first_symbol(HasRoot::False);
		let mut encountered = HashSet::new();
		let mut layouts = Vec::new();
		let mut combos = self.combos_ordered();

		for combo in combos.iter_mut().filter(|combo| !combo.is_empty()) {
			let first = ComboTreeElement::Symbol(get_first_symbol());
			let last = drain_last(combo);
			let mut layout = Vec::new();

			adjust_connections(&mut layouts, &mut layout, &first);
			layout.push(first);

			for (key_path, skill) in combo.drain(..) {
				let element = layout_element(key_path, skill, &mut encountered);
				adjust_connections(&mut layouts, &mut layout, &element);
				layout.push(element);
			}

			layout.push(last);

			layouts.push(layout);
		}

		layouts
	}
}

fn drain_last<TSkill, TId>(
	combo: &mut Vec<(Vec<SlotKey>, TSkill)>,
) -> ComboTreeElement<SlotKey, ComboSkill<TId>>
where
	TId: Debug + PartialEq + Clone,
	TSkill: GetProperty<SkillToken> + GetProperty<SkillIcon> + GetSkillId<TId>,
{
	let (key_path, skill) = combo.remove(combo.len() - 1);
	ComboTreeElement::Leaf {
		key_path,
		skill: ComboSkill::from(skill),
	}
}

#[derive(PartialEq)]
enum HasRoot {
	True,
	False,
}

fn get_first_symbol(mut has_root: HasRoot) -> impl FnMut() -> Symbol {
	move || {
		if has_root == HasRoot::False {
			has_root = HasRoot::True;
			return Symbol::Root;
		}

		Symbol::Corner
	}
}

fn adjust_connections<TKey, TSkill>(
	layouts: &mut [Vec<ComboTreeElement<TKey, TSkill>>],
	current_layout: &mut [ComboTreeElement<TKey, TSkill>],
	element: &ComboTreeElement<TKey, TSkill>,
) where
	TKey: PartialEq,
	TSkill: PartialEq,
{
	if element != &ComboTreeElement::Symbol(Symbol::Corner) {
		return;
	}

	if let Some((preceding_index, preceding)) = current_layout.iter_mut().enumerate().next_back() {
		*preceding = ComboTreeElement::Symbol(Symbol::Empty);
		replace_symbols_at(layouts, preceding_index, Symbol::Line, Symbol::Empty);
	};

	replace_symbols_at(layouts, current_layout.len(), Symbol::Empty, Symbol::Line);
}

fn layout_element<TSkill, TId>(
	key_path: Vec<SlotKey>,
	skill: TSkill,
	encountered: &mut HashSet<Vec<SlotKey>>,
) -> ComboTreeElement<SlotKey, ComboSkill<TId>>
where
	TSkill: GetSkillId<TId> + GetProperty<SkillToken> + GetProperty<SkillIcon>,
	TId: Debug + PartialEq + Clone,
{
	if encountered.contains(&key_path) {
		return ComboTreeElement::Symbol(Symbol::Corner);
	}

	encountered.insert(key_path.clone());

	ComboTreeElement::Node {
		key_path,
		skill: ComboSkill::from(skill),
	}
}

fn replace_symbols_at<TKey, TSkill>(
	layouts: &mut [Vec<ComboTreeElement<TKey, TSkill>>],
	index: usize,
	old: Symbol,
	new: Symbol,
) where
	TKey: PartialEq,
	TSkill: PartialEq,
{
	let elements = layouts
		.iter_mut()
		.rev()
		.filter_map(|layout| layout.get_mut(index))
		.take_while(|element| element == &&ComboTreeElement::Symbol(old));

	for element in elements {
		*element = ComboTreeElement::Symbol(new);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{
		tools::action_key::slot::{PlayerSlot, SlotKey},
		traits::{handles_loadout::combos::Combo, handles_localization::Token},
	};
	use testing::new_handle;
	use uuid::Uuid;

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill {
		id: Uuid,
		token: Token,
		icon: Handle<Image>,
	}

	impl _Skill {
		fn random_ids(token: &str) -> Self {
			Self {
				id: Uuid::new_v4(),
				token: Token::from(token),
				icon: new_handle(),
			}
		}
	}

	impl GetSkillId<Uuid> for _Skill {
		fn get_skill_id(&self) -> Uuid {
			self.id
		}
	}

	impl GetProperty<SkillToken> for _Skill {
		fn get_property(&self) -> &'_ Token {
			&self.token
		}
	}

	impl GetProperty<SkillIcon> for _Skill {
		fn get_property(&self) -> &'_ Handle<Image> {
			&self.icon
		}
	}

	struct _Combos(Vec<Combo<SlotKey, _Skill>>);

	impl GetCombosOrdered for _Combos {
		type TSkill = _Skill;

		fn combos_ordered(&self) -> Vec<Combo<SlotKey, _Skill>> {
			self.0.clone()
		}
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ l
	/// ```
	fn get_tree_leaf() {
		let skill = _Skill {
			id: Uuid::new_v4(),
			token: Token::from("a"),
			icon: new_handle(),
		};
		let combos = _Combos(vec![vec![(
			vec![SlotKey::from(PlayerSlot::LOWER_R)],
			skill.clone(),
		)]]);

		let tree = combos.build_combo_tree_layout();

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Leaf {
					key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
					skill: ComboSkill::from(skill),
				}
			]],
			tree
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n l
	/// ```
	fn get_tree_node_and_leaf() {
		let a = _Skill::random_ids("a");
		let b = _Skill::random_ids("b");
		let combos = _Combos(vec![vec![
			(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
			(
				vec![
					SlotKey::from(PlayerSlot::LOWER_R),
					SlotKey::from(PlayerSlot::LOWER_R),
				],
				b.clone(),
			),
		]]);

		let tree = combos.build_combo_tree_layout();

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Node {
					key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
					skill: ComboSkill::from(a),
				},
				ComboTreeElement::Leaf {
					key_path: vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R)
					],
					skill: ComboSkill::from(b),
				}
			]],
			tree
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ l
	/// └ l
	/// ```
	fn layout_two_combos_with_one_skill() {
		let a = _Skill::random_ids("a");
		let b = _Skill::random_ids("b");
		let combos = _Combos(vec![
			vec![(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone())],
			vec![(vec![SlotKey::from(PlayerSlot::LOWER_L)], b.clone())],
		]);

		let tree = combos.build_combo_tree_layout();

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: ComboSkill::from(a),
					}
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: ComboSkill::from(b),
					}
				]
			],
			tree,
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n l
	///   └ l
	/// ```
	fn layout_two_combos_with_two_skills_where_one_matches() {
		let a = _Skill::random_ids("a");
		let b = _Skill::random_ids("b");
		let c = _Skill::random_ids("c");
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					b.clone(),
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					c.clone(),
				),
			],
		]);

		let tree = combos.build_combo_tree_layout();

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: ComboSkill::from(a),
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: ComboSkill::from(b),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L)
						],
						skill: ComboSkill::from(c),
					},
				]
			],
			tree,
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n l
	/// │ └ l
	/// └ l
	/// ```
	fn layout_three_combos_with_complex_setup() {
		let a = _Skill::random_ids("a");
		let b = _Skill::random_ids("b");
		let c = _Skill::random_ids("c");
		let d = _Skill::random_ids("d");
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					b.clone(),
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					c.clone(),
				),
			],
			vec![(vec![SlotKey::from(PlayerSlot::LOWER_L)], d.clone())],
		]);

		let tree = combos.build_combo_tree_layout();

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: ComboSkill::from(a),
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						skill: ComboSkill::from(b),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L)
						],
						skill: ComboSkill::from(c),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: ComboSkill::from(d),
					},
				],
			],
			tree,
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n n l
	/// │ │ └ l
	/// │ └ l
	/// └ l
	/// ```
	fn layout_three_combos_with_complex_setup_2() {
		let a = _Skill::random_ids("a");
		let b = _Skill::random_ids("b");
		let c = _Skill::random_ids("c");
		let d = _Skill::random_ids("d");
		let e = _Skill::random_ids("e");
		let f = _Skill::random_ids("f");
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					b.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					c.clone(),
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					b.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					d.clone(),
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					e.clone(),
				),
			],
			vec![(vec![SlotKey::from(PlayerSlot::LOWER_L)], f.clone())],
		]);

		let tree = combos.build_combo_tree_layout();

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: ComboSkill::from(a),
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: ComboSkill::from(b),
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: ComboSkill::from(c),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L)
						],
						skill: ComboSkill::from(d),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L)
						],
						skill: ComboSkill::from(e),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: ComboSkill::from(f),
					},
				],
			],
			tree,
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n n l
	///   │ └ l
	///   └ l
	/// ```
	fn layout_three_combos_with_complex_setup_3() {
		let a = _Skill::random_ids("a");
		let b = _Skill::random_ids("b");
		let c = _Skill::random_ids("c");
		let d = _Skill::random_ids("d");
		let e = _Skill::random_ids("e");
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					b.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					c.clone(),
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					b.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					d.clone(),
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					e.clone(),
				),
			],
		]);

		let tree = combos.build_combo_tree_layout();

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: ComboSkill::from(a),
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: ComboSkill::from(b),
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: ComboSkill::from(c),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L)
						],
						skill: ComboSkill::from(d),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L)
						],
						skill: ComboSkill::from(e),
					},
				],
			],
			tree,
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n n n l
	///   │ └ n l
	///   │   └ l
	///   └ l
	/// ```
	fn layout_three_combos_with_complex_setup_4() {
		let a = _Skill::random_ids("a");
		let b = _Skill::random_ids("b");
		let c = _Skill::random_ids("c");
		let d = _Skill::random_ids("d");
		let e = _Skill::random_ids("e");
		let f = _Skill::random_ids("f");
		let g = _Skill::random_ids("g");
		let h = _Skill::random_ids("h");
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					b.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					c.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					d.clone(),
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					b.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					e.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					f.clone(),
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					b.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					e.clone(),
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					g.clone(),
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], a.clone()),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					h.clone(),
				),
			],
		]);

		let tree = combos.build_combo_tree_layout();

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: ComboSkill::from(a),
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: ComboSkill::from(b),
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						skill: ComboSkill::from(c),
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						skill: ComboSkill::from(d),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L),
						],
						skill: ComboSkill::from(e),
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						skill: ComboSkill::from(f),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L),
							SlotKey::from(PlayerSlot::LOWER_L),
						],
						skill: ComboSkill::from(g),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L),
						],
						skill: ComboSkill::from(h),
					},
				]
			],
			tree,
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n l
	/// │ └ l
	/// └ n l
	///   └ l
	/// ```
	fn layout_combos_with_no_broken_lines() {
		let a = _Skill::random_ids("a");
		let b = _Skill::random_ids("b");
		let c = _Skill::random_ids("c");
		let d = _Skill::random_ids("d");
		let e = _Skill::random_ids("e");
		let f = _Skill::random_ids("f");
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_L)], a.clone()),
				(vec![SlotKey::from(PlayerSlot::LOWER_L)], b.clone()),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_L)], a.clone()),
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], c.clone()),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], d.clone()),
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], e.clone()),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], d.clone()),
				(vec![SlotKey::from(PlayerSlot::LOWER_L)], f.clone()),
			],
		]);

		let tree = combos.build_combo_tree_layout();

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: ComboSkill::from(a),
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: ComboSkill::from(b),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: ComboSkill::from(c),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: ComboSkill::from(d),
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: ComboSkill::from(e),
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: ComboSkill::from(f),
					},
				],
			],
			tree,
		);
	}
}
