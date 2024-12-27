use common::tools::slot_key::SlotKey;
use skills::{skills::Skill, traits::GetCombosOrdered};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Symbol {
	Root,
	Line,
	Corner,
	Empty,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum ComboTreeElement {
	Node {
		key_path: Vec<SlotKey>,
		skill: Skill,
	},
	Leaf {
		key_path: Vec<SlotKey>,
		skill: Skill,
	},
	Symbol(Symbol),
}

pub type ComboTreeLayout = Vec<Vec<ComboTreeElement>>;

pub(crate) trait GetComboTreeLayout {
	fn combo_tree_layout(&self) -> ComboTreeLayout;
}

impl<T> GetComboTreeLayout for T
where
	T: GetCombosOrdered,
{
	fn combo_tree_layout(&self) -> ComboTreeLayout {
		let mut get_first_symbol = get_first_symbol(HasRoot::False);
		let mut encountered = HashSet::new();
		let mut layouts = Vec::new();

		for mut combo in self.combos_ordered().filter(|combo| !combo.is_empty()) {
			let first = ComboTreeElement::Symbol(get_first_symbol());
			let last = drain_as_leaf(&mut combo);
			let mut layout = Vec::new();

			adjust_connections(&mut layouts, &mut layout, &first);
			layout.push(first);

			for (key_path, skill) in combo.into_iter() {
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

fn drain_as_leaf(combo: &mut Vec<(Vec<SlotKey>, &Skill)>) -> ComboTreeElement {
	let leaf = combo.remove(combo.len() - 1);
	ComboTreeElement::Leaf {
		key_path: leaf.0,
		skill: leaf.1.clone(),
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

fn adjust_connections(
	layouts: &mut [Vec<ComboTreeElement>],
	current_layout: &mut [ComboTreeElement],
	element: &ComboTreeElement,
) {
	if element != &ComboTreeElement::Symbol(Symbol::Corner) {
		return;
	}

	if let Some((preceding_index, preceding)) = current_layout.iter_mut().enumerate().last() {
		*preceding = ComboTreeElement::Symbol(Symbol::Empty);
		replace_symbols_at(layouts, preceding_index, Symbol::Line, Symbol::Empty);
	};

	replace_symbols_at(layouts, current_layout.len(), Symbol::Empty, Symbol::Line);
}

fn layout_element(
	key_path: Vec<SlotKey>,
	skill: &Skill,
	encountered: &mut HashSet<Vec<SlotKey>>,
) -> ComboTreeElement {
	if encountered.contains(&key_path) {
		return ComboTreeElement::Symbol(Symbol::Corner);
	}

	encountered.insert(key_path.clone());

	ComboTreeElement::Node {
		key_path,
		skill: skill.clone(),
	}
}

fn replace_symbols_at(
	layouts: &mut [Vec<ComboTreeElement>],
	index: usize,
	old: Symbol,
	new: Symbol,
) {
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
	use common::tools::slot_key::Side;
	use skills::traits::Combo;

	struct _Combos(Vec<Vec<(Vec<SlotKey>, Skill)>>);

	impl GetCombosOrdered for _Combos {
		fn combos_ordered(&self) -> impl Iterator<Item = Combo> {
			self.0.iter().map(|combo| {
				combo
					.iter()
					.map(|(key_path, skill)| (key_path.clone(), skill))
					.collect()
			})
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
			Skill::default(),
		)]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Leaf {
					key_path: vec![SlotKey::BottomHand(Side::Right)],
					skill: Skill::default()
				}
			]],
			combos.combo_tree_layout()
		);
	}

	#[test]
	/// **layout** *root (◯), node (n), leaf (l)*:
	/// ```
	/// ◯ n l
	/// ```
	fn get_tree_node_and_leaf() {
		let combos = _Combos(vec![vec![
			(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
			(
				vec![
					SlotKey::BottomHand(Side::Right),
					SlotKey::BottomHand(Side::Right),
				],
				Skill::default(),
			),
		]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Node {
					key_path: vec![SlotKey::BottomHand(Side::Right)],
					skill: Skill::default()
				},
				ComboTreeElement::Leaf {
					key_path: vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right)
					],
					skill: Skill::default()
				}
			]],
			combos.combo_tree_layout()
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
			vec![(vec![SlotKey::BottomHand(Side::Right)], Skill::default())],
			vec![(vec![SlotKey::BottomHand(Side::Left)], Skill::default())],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: Skill::default()
					}
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: Skill::default()
					}
				]
			],
			combos.combo_tree_layout()
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
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: Skill::default()
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: Skill::default()
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
						skill: Skill::default()
					},
				]
			],
			combos.combo_tree_layout()
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
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
			],
			vec![(vec![SlotKey::BottomHand(Side::Left)], Skill::default())],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: Skill::default()
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						skill: Skill::default()
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
						skill: Skill::default()
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: Skill::default()
					},
				],
			],
			combos.combo_tree_layout()
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
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
			],
			vec![(vec![SlotKey::BottomHand(Side::Left)], Skill::default())],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: Skill::default()
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: Skill::default()
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: Skill::default()
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
						skill: Skill::default()
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
						skill: Skill::default()
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: Skill::default()
					},
				],
			],
			combos.combo_tree_layout()
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
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: Skill::default()
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: Skill::default()
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: Skill::default()
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
						skill: Skill::default()
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
						skill: Skill::default()
					},
				],
			],
			combos.combo_tree_layout()
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
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					Skill::default(),
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: Skill::default()
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						skill: Skill::default()
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						skill: Skill::default()
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						skill: Skill::default()
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
						skill: Skill::default()
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
							SlotKey::BottomHand(Side::Right),
						],
						skill: Skill::default()
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
						skill: Skill::default()
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
						skill: Skill::default()
					},
				]
			],
			combos.combo_tree_layout()
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
				(vec![SlotKey::BottomHand(Side::Left)], Skill::default()),
				(vec![SlotKey::BottomHand(Side::Left)], Skill::default()),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Left)], Skill::default()),
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
			],
			vec![
				(vec![SlotKey::BottomHand(Side::Right)], Skill::default()),
				(vec![SlotKey::BottomHand(Side::Left)], Skill::default()),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: Skill::default()
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: Skill::default()
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: Skill::default()
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: Skill::default()
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						skill: Skill::default()
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						skill: Skill::default()
					},
				],
			],
			combos.combo_tree_layout()
		);
	}
}
