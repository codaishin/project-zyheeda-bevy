use bevy::prelude::*;
use common::{
	tools::action_key::slot::PlayerSlot,
	traits::{
		accessors::get::RefInto,
		handles_combo_menu::GetCombosOrdered,
		handles_localization::Token,
	},
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
		key_path: Vec<PlayerSlot>,
		skill: TSkill,
	},
	Leaf {
		key_path: Vec<PlayerSlot>,
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
	T: GetCombosOrdered<TSkill, PlayerSlot>,
	TSkill: Clone
		+ PartialEq
		+ for<'a> RefInto<'a, &'a Token>
		+ for<'a> RefInto<'a, &'a Option<Handle<Image>>>,
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

fn drain<TSkill>(combo: &mut Vec<(Vec<PlayerSlot>, TSkill)>) -> ComboTreeElement<TSkill> {
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
	key_path: Vec<PlayerSlot>,
	skill: TSkill,
	encountered: &mut HashSet<Vec<PlayerSlot>>,
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
	use super::*;
	use common::{
		tools::action_key::slot::Side,
		traits::{handles_combo_menu::Combo, handles_localization::Token},
	};
	use std::sync::LazyLock;

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill(Option<Handle<Image>>);

	static TOKEN: LazyLock<Token> = LazyLock::new(|| Token::from(""));

	impl<'a> RefInto<'a, &'a Token> for _Skill {
		fn ref_into(&self) -> &Token {
			&TOKEN
		}
	}

	impl<'a> RefInto<'a, &'a Option<Handle<Image>>> for _Skill {
		fn ref_into(&self) -> &Option<Handle<Image>> {
			&self.0
		}
	}

	struct _Combos(Vec<Combo<PlayerSlot, _Skill>>);

	impl GetCombosOrdered<_Skill, PlayerSlot> for _Combos {
		fn combos_ordered(&self) -> Vec<Combo<PlayerSlot, _Skill>> {
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
			vec![PlayerSlot::Lower(Side::Right)],
			_Skill(None),
		)]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Leaf {
					key_path: vec![PlayerSlot::Lower(Side::Right)],
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
			(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
			(
				vec![
					PlayerSlot::Lower(Side::Right),
					PlayerSlot::Lower(Side::Right),
				],
				_Skill(None),
			),
		]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Node {
					key_path: vec![PlayerSlot::Lower(Side::Right)],
					skill: _Skill(None)
				},
				ComboTreeElement::Leaf {
					key_path: vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right)
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
			vec![(vec![PlayerSlot::Lower(Side::Right)], _Skill(None))],
			vec![(vec![PlayerSlot::Lower(Side::Left)], _Skill(None))],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Leaf {
						key_path: vec![PlayerSlot::Lower(Side::Right)],
						skill: _Skill(None)
					}
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![PlayerSlot::Lower(Side::Left)],
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
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
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
						key_path: vec![PlayerSlot::Lower(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left)
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
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![(vec![PlayerSlot::Lower(Side::Left)], _Skill(None))],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![PlayerSlot::Lower(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![PlayerSlot::Lower(Side::Left)],
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
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![(vec![PlayerSlot::Lower(Side::Left)], _Skill(None))],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![PlayerSlot::Lower(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Node {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
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
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![PlayerSlot::Lower(Side::Left)],
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
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
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
						key_path: vec![PlayerSlot::Lower(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Node {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
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
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left)
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left)
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
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
					],
					_Skill(None),
				),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
						PlayerSlot::Lower(Side::Left),
					],
					_Skill(None),
				),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(
					vec![
						PlayerSlot::Lower(Side::Right),
						PlayerSlot::Lower(Side::Left),
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
						key_path: vec![PlayerSlot::Lower(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Node {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right)
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Node {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
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
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left),
						],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left),
							PlayerSlot::Lower(Side::Right),
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
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left),
							PlayerSlot::Lower(Side::Left),
						],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![
							PlayerSlot::Lower(Side::Right),
							PlayerSlot::Lower(Side::Left),
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
				(vec![PlayerSlot::Lower(Side::Left)], _Skill(None)),
				(vec![PlayerSlot::Lower(Side::Left)], _Skill(None)),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Left)], _Skill(None)),
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
			],
			vec![
				(vec![PlayerSlot::Lower(Side::Right)], _Skill(None)),
				(vec![PlayerSlot::Lower(Side::Left)], _Skill(None)),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![PlayerSlot::Lower(Side::Left)],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![PlayerSlot::Lower(Side::Left)],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![PlayerSlot::Lower(Side::Right)],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Node {
						key_path: vec![PlayerSlot::Lower(Side::Right)],
						skill: _Skill(None)
					},
					ComboTreeElement::Leaf {
						key_path: vec![PlayerSlot::Lower(Side::Right)],
						skill: _Skill(None)
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![PlayerSlot::Lower(Side::Left)],
						skill: _Skill(None)
					},
				],
			],
			combos.build_combo_tree_layout()
		);
	}
}
