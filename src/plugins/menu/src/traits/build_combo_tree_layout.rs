use common::{
	tools::{action_key::slot::SlotKey, skill_description::SkillToken, skill_icon::SkillIcon},
	traits::{handles_combo_menu::GetCombosOrdered, inspect_able::InspectAble},
};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Symbol {
	Root,
	Line,
	Corner,
	Empty,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ComboTreeElement<TSkill> {
	Node {
		key_path: Vec<SlotKey>,
		skill: TSkill,
	},
	Leaf {
		key_path: Vec<SlotKey>,
		skill: TSkill,
	},
	Symbol(Symbol),
}

pub type ComboTreeLayout<TSkill> = Vec<Vec<ComboTreeElement<TSkill>>>;

pub(crate) trait BuildComboTreeLayout<TSkill> {
	fn build_combo_tree_layout(&self) -> ComboTreeLayout<TSkill>;
}

impl<T, TSkill> BuildComboTreeLayout<TSkill> for T
where
	T: GetCombosOrdered<TSkill>,
	TSkill: InspectAble<SkillToken> + InspectAble<SkillIcon> + Clone + PartialEq,
{
	fn build_combo_tree_layout(&self) -> ComboTreeLayout<TSkill> {
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

fn drain<TSkill>(combo: &mut Vec<(Vec<SlotKey>, TSkill)>) -> ComboTreeElement<TSkill> {
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

fn adjust_connections<TSkill>(
	layouts: &mut [Vec<ComboTreeElement<TSkill>>],
	current_layout: &mut [ComboTreeElement<TSkill>],
	element: &ComboTreeElement<TSkill>,
) where
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

fn layout_element<TSkill>(
	key_path: Vec<SlotKey>,
	skill: TSkill,
	encountered: &mut HashSet<Vec<SlotKey>>,
) -> ComboTreeElement<TSkill> {
	if encountered.contains(&key_path) {
		return ComboTreeElement::Symbol(Symbol::Corner);
	}

	encountered.insert(key_path.clone());

	ComboTreeElement::Node {
		key_path: key_path.clone(),
		skill,
	}
}

fn replace_symbols_at<TSkill>(
	layouts: &mut [Vec<ComboTreeElement<TSkill>>],
	index: usize,
	old: Symbol,
	new: Symbol,
) where
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
	use std::sync::LazyLock;

	use super::*;
	use bevy::prelude::*;
	use common::{
		tools::action_key::slot::{Combo, Side},
		traits::handles_localization::Token,
	};

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill(Option<Handle<Image>>);

	static TOKEN: LazyLock<Token> = LazyLock::new(|| Token::from(""));

	impl InspectAble<SkillToken> for _Skill {
		fn get_inspect_able_field(&self) -> &Token {
			&TOKEN
		}
	}

	impl InspectAble<SkillIcon> for _Skill {
		fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
			&self.0
		}
	}

	struct _Combos(Vec<Combo<_Skill>>);

	impl GetCombosOrdered<_Skill> for _Combos {
		fn combos_ordered(&self) -> Vec<Combo<_Skill>> {
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
			vec![SlotKey::BottomHand(Side::Right)],
			_Skill(None),
		)]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Leaf {
					key_path: vec![SlotKey::BottomHand(Side::Right)],
					skill: _Skill(None)
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
			(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
			(
				vec![
					SlotKey::BottomHand(Side::Right),
					SlotKey::BottomHand(Side::Right),
				],
				_Skill(None),
			),
		]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Node {
					key_path: vec![SlotKey::BottomHand(Side::Right)],
					skill: _Skill(None)
				},
				ComboTreeElement::Leaf {
					key_path: vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right)
					],
					skill: _Skill(None)
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
			vec![(vec![SlotKey::BottomHand(Side::Right)], _Skill(None))],
			vec![(vec![SlotKey::BottomHand(Side::Left)], _Skill(None))],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: _Skill(None)
					}
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: _Skill(None)
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
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left)
						],
						skill: _Skill(None)
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
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![(vec![SlotKey::BottomHand(Side::Left)], _Skill(None))],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: _Skill(None)
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
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![(vec![SlotKey::BottomHand(Side::Left)], _Skill(None))],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: _Skill(None)
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
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left)
						],
						skill: _Skill(None)
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
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					_Skill(None),
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
							SlotKey::BottomHand(Side::Right),
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
							SlotKey::BottomHand(Side::Left),
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
						],
						skill: _Skill(None)
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
				(vec![SlotKey::BottomHand(Side::Left)], _Skill(None)),
				(vec![SlotKey::BottomHand(Side::Left)], _Skill(None)),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Left)], _Skill(None)),
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], _Skill(None)),
				(vec![SlotKey::BottomHand(Side::Left)], _Skill(None)),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: _Skill(None)
					},
				],
			],
			combos.build_combo_tree_layout()
		);
	}
}
