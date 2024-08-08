use bevy::prelude::default;
use skills::{items::slot_key::SlotKey, skills::Skill, traits::GetCombosOrdered};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Symbol {
	Root,
	Line,
	Corner,
	Empty,
}

#[derive(Debug, PartialEq)]
pub(crate) enum SkillTreeElement<'a> {
	Node {
		key_path: Vec<SlotKey>,
		skill: &'a Skill,
	},
	Leaf {
		key_path: Vec<SlotKey>,
		skill: &'a Skill,
	},
	Symbol(Symbol),
}

pub type ComboLayout<'a> = Vec<SkillTreeElement<'a>>;

pub(crate) trait ComboTreeLayout {
	fn combo_tree_layout(&self) -> Vec<ComboLayout>;
}

impl<T> ComboTreeLayout for T
where
	T: GetCombosOrdered,
{
	fn combo_tree_layout(&self) -> Vec<ComboLayout> {
		let mut get_first_symbol = get_first_symbol(HasRoot::False);
		let mut encountered = HashSet::new();
		let mut layouts = Vec::new();

		for mut combo in self.combos_ordered().filter(|combo| !combo.is_empty()) {
			let first = SkillTreeElement::Symbol(get_first_symbol());
			let last = drain_as_leaf(&mut combo);
			let mut layout = Vec::new();

			adjust_connections(&mut layouts, &mut layout, &first, LayoutIndex(0));
			layout.push(first);

			for (i, (key_path, skill)) in combo.into_iter().enumerate() {
				let element = layout_element(key_path, skill, &mut encountered);
				adjust_connections(&mut layouts, &mut layout, &element, LayoutIndex(i + 1));
				layout.push(element);
			}

			layout.push(last);

			layouts.push(layout);
		}

		layouts
	}
}

fn drain_as_leaf<'a>(combo: &mut Vec<(Vec<SlotKey>, &'a Skill)>) -> SkillTreeElement<'a> {
	let leaf = combo.remove(combo.len() - 1);
	SkillTreeElement::Leaf {
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

struct LayoutIndex(usize);

impl LayoutIndex {
	fn current(&self) -> usize {
		self.0
	}

	fn preceding(&self) -> Option<usize> {
		self.0.checked_sub(1)
	}
}

fn adjust_connections(
	layouts: &mut [Vec<SkillTreeElement>],
	current_layout: &mut [SkillTreeElement],
	element: &SkillTreeElement,
	index: LayoutIndex,
) {
	if element != &SkillTreeElement::Symbol(Symbol::Corner) {
		return;
	}

	if let Some(preceding) = current_layout.last_mut() {
		*preceding = SkillTreeElement::Symbol(Symbol::Empty);
	};

	if let Some(index) = index.preceding() {
		replace_in_previous(layouts, index, Symbol::Line, Symbol::Empty);
	}

	replace_in_previous(layouts, index.current(), Symbol::Empty, Symbol::Line);
}

fn layout_element<'a>(
	key_path: Vec<SlotKey>,
	skill: &'a Skill,
	encountered: &mut HashSet<Vec<SlotKey>>,
) -> SkillTreeElement<'a> {
	if encountered.contains(&key_path) {
		return SkillTreeElement::Symbol(Symbol::Corner);
	}

	encountered.insert(key_path.clone());

	SkillTreeElement::Node { key_path, skill }
}

fn replace_in_previous(
	layouts: &mut [Vec<SkillTreeElement>],
	index: usize,
	old: Symbol,
	new: Symbol,
) {
	let elements = layouts
		.iter_mut()
		.filter_map(|layout| layout.get_mut(index))
		.filter(|element| element == &&SkillTreeElement::Symbol(old));

	for element in elements {
		*element = SkillTreeElement::Symbol(new);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::components::Side;
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
			vec![SlotKey::Hand(Side::Main)],
			Skill::default(),
		)]]);

		assert_eq!(
			vec![vec![
				SkillTreeElement::Symbol(Symbol::Root),
				SkillTreeElement::Leaf {
					key_path: vec![SlotKey::Hand(Side::Main)],
					skill: &Skill::default()
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
			(vec![SlotKey::Hand(Side::Main)], Skill::default()),
			(
				vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
				Skill::default(),
			),
		]]);

		assert_eq!(
			vec![vec![
				SkillTreeElement::Symbol(Symbol::Root),
				SkillTreeElement::Node {
					key_path: vec![SlotKey::Hand(Side::Main)],
					skill: &Skill::default()
				},
				SkillTreeElement::Leaf {
					key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					skill: &Skill::default()
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
			vec![(vec![SlotKey::Hand(Side::Main)], Skill::default())],
			vec![(vec![SlotKey::Hand(Side::Off)], Skill::default())],
		]);

		assert_eq!(
			vec![
				vec![
					SkillTreeElement::Symbol(Symbol::Root),
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					}
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Off)],
						skill: &Skill::default()
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
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
					Skill::default(),
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					SkillTreeElement::Symbol(Symbol::Root),
					SkillTreeElement::Node {
						key_path: vec![SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					},
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Empty),
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
						skill: &Skill::default()
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
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
					Skill::default(),
				),
			],
			vec![(vec![SlotKey::Hand(Side::Off)], Skill::default())],
		]);

		assert_eq!(
			vec![
				vec![
					SkillTreeElement::Symbol(Symbol::Root),
					SkillTreeElement::Node {
						key_path: vec![SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					},
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main),],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Line),
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Off)],
						skill: &Skill::default()
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
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Off),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
					Skill::default(),
				),
			],
			vec![(vec![SlotKey::Hand(Side::Off)], Skill::default())],
		]);

		assert_eq!(
			vec![
				vec![
					SkillTreeElement::Symbol(Symbol::Root),
					SkillTreeElement::Node {
						key_path: vec![SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					},
					SkillTreeElement::Node {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					},
					SkillTreeElement::Leaf {
						key_path: vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main)
						],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Line),
					SkillTreeElement::Symbol(Symbol::Line),
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Off)
						],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Line),
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Off)],
						skill: &Skill::default()
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
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Off),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
					Skill::default(),
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					SkillTreeElement::Symbol(Symbol::Root),
					SkillTreeElement::Node {
						key_path: vec![SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					},
					SkillTreeElement::Node {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					},
					SkillTreeElement::Leaf {
						key_path: vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main)
						],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Empty),
					SkillTreeElement::Symbol(Symbol::Line),
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Off)
						],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Empty),
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
						skill: &Skill::default()
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
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Off),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Off),
						SlotKey::Hand(Side::Main),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Off),
					],
					Skill::default(),
				),
				(
					vec![
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Main),
						SlotKey::Hand(Side::Off),
						SlotKey::Hand(Side::Off),
					],
					Skill::default(),
				),
			],
			vec![
				(vec![SlotKey::Hand(Side::Main)], Skill::default()),
				(
					vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)],
					Skill::default(),
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					SkillTreeElement::Symbol(Symbol::Root),
					SkillTreeElement::Node {
						key_path: vec![SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					},
					SkillTreeElement::Node {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Main)],
						skill: &Skill::default()
					},
					SkillTreeElement::Node {
						key_path: vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
						],
						skill: &Skill::default()
					},
					SkillTreeElement::Leaf {
						key_path: vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
						],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Empty),
					SkillTreeElement::Symbol(Symbol::Line),
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Node {
						key_path: vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Off),
						],
						skill: &Skill::default()
					},
					SkillTreeElement::Leaf {
						key_path: vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Off),
							SlotKey::Hand(Side::Main),
						],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Empty),
					SkillTreeElement::Symbol(Symbol::Line),
					SkillTreeElement::Symbol(Symbol::Empty),
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Main),
							SlotKey::Hand(Side::Off),
							SlotKey::Hand(Side::Off),
						],
						skill: &Skill::default()
					},
				],
				vec![
					SkillTreeElement::Symbol(Symbol::Empty),
					SkillTreeElement::Symbol(Symbol::Corner),
					SkillTreeElement::Leaf {
						key_path: vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off),],
						skill: &Skill::default()
					},
				]
			],
			combos.combo_tree_layout()
		);
	}
}
