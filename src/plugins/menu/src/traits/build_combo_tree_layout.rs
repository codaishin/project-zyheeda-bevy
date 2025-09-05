use bevy::prelude::*;
use common::traits::handles_loadout::{
	combos_component::GetCombosOrdered,
	loadout::{LoadoutItem, LoadoutKey},
};
use std::{collections::HashSet, hash::Hash};

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

pub type ComboTreeLayout<TKey, TSkill> = Vec<Vec<ComboTreeElement<TKey, TSkill>>>;

pub(crate) trait BuildComboTreeLayout: LoadoutItem + LoadoutKey {
	fn build_combo_tree_layout(&self) -> ComboTreeLayout<Self::TKey, Self::TItem>;
}

impl<T> BuildComboTreeLayout for T
where
	T: GetCombosOrdered,
	T::TKey: Eq + Hash + Copy,
	T::TItem: Clone + PartialEq,
{
	fn build_combo_tree_layout(&self) -> ComboTreeLayout<Self::TKey, Self::TItem> {
		let mut get_first_symbol = get_first_symbol(HasRoot::False);
		let mut encountered = HashSet::new();
		let mut layouts = Vec::new();
		let combos = self.combos_ordered();

		for mut combo in combos.iter().filter(|combo| !combo.is_empty()).cloned() {
			let first = ComboTreeElement::Symbol(get_first_symbol());
			let last = drain(&mut combo);
			let mut layout = Vec::new();

			adjust_connections(&mut layouts, &mut layout, &first);
			layout.push(first);

			for (key_path, skill) in combo.into_iter() {
				let element = layout_element(key_path, skill.clone(), &mut encountered);
				adjust_connections(&mut layouts, &mut layout, &element);
				layout.push(element);
			}

			layout.push(last);

			layouts.push(layout);
		}

		layouts
	}
}

fn drain<TKey, TSkill>(combo: &mut Vec<(Vec<TKey>, TSkill)>) -> ComboTreeElement<TKey, TSkill> {
	let leaf = combo.remove(combo.len() - 1);
	ComboTreeElement::Leaf {
		key_path: leaf.0,
		skill: leaf.1,
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

fn layout_element<TKey, TSkill>(
	key_path: Vec<TKey>,
	skill: TSkill,
	encountered: &mut HashSet<Vec<TKey>>,
) -> ComboTreeElement<TKey, TSkill>
where
	TKey: Eq + Hash + Copy,
{
	if encountered.contains(&key_path) {
		return ComboTreeElement::Symbol(Symbol::Corner);
	}

	encountered.insert(key_path.clone());

	ComboTreeElement::Node {
		key_path: key_path.clone(),
		skill,
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
		traits::handles_loadout::combos_component::Combo,
	};

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill;

	struct _Combos(Vec<Combo<SlotKey, _Skill>>);

	impl LoadoutKey for _Combos {
		type TKey = SlotKey;
	}

	impl LoadoutItem for _Combos {
		type TItem = _Skill;
	}

	impl GetCombosOrdered for _Combos {
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
		let combos = _Combos(vec![vec![(
			vec![SlotKey::from(PlayerSlot::LOWER_R)],
			_Skill,
		)]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Leaf {
					key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
					skill: _Skill
				}
			]],
			combos.build_combo_tree_layout()
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n l
	/// ```
	fn get_tree_node_and_leaf() {
		let combos = _Combos(vec![vec![
			(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
			(
				vec![
					SlotKey::from(PlayerSlot::LOWER_R),
					SlotKey::from(PlayerSlot::LOWER_R),
				],
				_Skill,
			),
		]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Node {
					key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
					skill: _Skill
				},
				ComboTreeElement::Leaf {
					key_path: vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R)
					],
					skill: _Skill
				}
			]],
			combos.build_combo_tree_layout()
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ l
	/// └ l
	/// ```
	fn layout_two_combos_with_one_skill() {
		let combos = _Combos(vec![
			vec![(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill)],
			vec![(vec![SlotKey::from(PlayerSlot::LOWER_L)], _Skill)],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: _Skill
					}
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: _Skill
					}
				]
			],
			combos.build_combo_tree_layout()
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n l
	///   └ l
	/// ```
	fn layout_two_combos_with_two_skills_where_one_matches() {
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: _Skill
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: _Skill
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
						skill: _Skill
					},
				]
			],
			combos.build_combo_tree_layout()
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
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
			],
			vec![(vec![SlotKey::from(PlayerSlot::LOWER_L)], _Skill)],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: _Skill
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						skill: _Skill
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
						skill: _Skill
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: _Skill
					},
				],
			],
			combos.build_combo_tree_layout()
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
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
			],
			vec![(vec![SlotKey::from(PlayerSlot::LOWER_L)], _Skill)],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: _Skill
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: _Skill
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: _Skill
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
						skill: _Skill
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
						skill: _Skill
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: _Skill
					},
				],
			],
			combos.build_combo_tree_layout()
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
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: _Skill
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: _Skill
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: _Skill
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
						skill: _Skill
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
						skill: _Skill
					},
				],
			],
			combos.build_combo_tree_layout()
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
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(
					vec![
						SlotKey::from(PlayerSlot::LOWER_R),
						SlotKey::from(PlayerSlot::LOWER_L),
					],
					_Skill,
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: _Skill
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R)
						],
						skill: _Skill
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						skill: _Skill
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						skill: _Skill
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
						skill: _Skill
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_R),
							SlotKey::from(PlayerSlot::LOWER_L),
							SlotKey::from(PlayerSlot::LOWER_R),
						],
						skill: _Skill
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
						skill: _Skill
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
						skill: _Skill
					},
				]
			],
			combos.build_combo_tree_layout()
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
		let combos = _Combos(vec![
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_L)], _Skill),
				(vec![SlotKey::from(PlayerSlot::LOWER_L)], _Skill),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_L)], _Skill),
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
			],
			vec![
				(vec![SlotKey::from(PlayerSlot::LOWER_R)], _Skill),
				(vec![SlotKey::from(PlayerSlot::LOWER_L)], _Skill),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: _Skill
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: _Skill
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: _Skill
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: _Skill
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_R)],
						skill: _Skill
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::from(PlayerSlot::LOWER_L)],
						skill: _Skill
					},
				],
			],
			combos.build_combo_tree_layout()
		);
	}
}
