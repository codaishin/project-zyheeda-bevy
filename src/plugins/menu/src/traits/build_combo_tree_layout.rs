use common::{
	tools::slot_key::SlotKey,
	traits::handles_combo_menu::{ComboSkillDescriptor, GetCombosOrdered},
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
		descriptor: ComboSkillDescriptor<TSkill>,
	},
	Leaf {
		key_path: Vec<SlotKey>,
		descriptor: ComboSkillDescriptor<TSkill>,
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
	TSkill: Clone + PartialEq,
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

fn drain<TSkill>(
	combo: &mut Vec<(Vec<SlotKey>, ComboSkillDescriptor<TSkill>)>,
) -> ComboTreeElement<TSkill> {
	let leaf = combo.remove(combo.len() - 1);
	ComboTreeElement::Leaf {
		key_path: leaf.0,
		descriptor: leaf.1,
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

	if let Some((preceding_index, preceding)) = current_layout.iter_mut().enumerate().last() {
		*preceding = ComboTreeElement::Symbol(Symbol::Empty);
		replace_symbols_at(layouts, preceding_index, Symbol::Line, Symbol::Empty);
	};

	replace_symbols_at(layouts, current_layout.len(), Symbol::Empty, Symbol::Line);
}

fn layout_element<TSkill>(
	key_path: Vec<SlotKey>,
	skill: ComboSkillDescriptor<TSkill>,
	encountered: &mut HashSet<Vec<SlotKey>>,
) -> ComboTreeElement<TSkill> {
	if encountered.contains(&key_path) {
		return ComboTreeElement::Symbol(Symbol::Corner);
	}

	encountered.insert(key_path.clone());

	ComboTreeElement::Node {
		key_path: key_path.clone(),
		descriptor: skill,
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
	use bevy::utils::default;
	use common::{tools::slot_key::Side, traits::handles_equipment::Combo};

	#[derive(Debug, PartialEq, Default, Clone)]
	struct _Skill;

	struct _Combos(Vec<Combo<ComboSkillDescriptor<_Skill>>>);

	impl GetCombosOrdered<_Skill> for _Combos {
		fn combos_ordered(&self) -> Vec<Combo<ComboSkillDescriptor<_Skill>>> {
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
			ComboSkillDescriptor {
				skill: _Skill,
				..default()
			},
		)]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Leaf {
					key_path: vec![SlotKey::BottomHand(Side::Right)],
					descriptor: ComboSkillDescriptor {
						skill: _Skill,
						..default()
					}
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
			(
				vec![SlotKey::BottomHand(Side::Right)],
				ComboSkillDescriptor {
					skill: _Skill,
					..default()
				},
			),
			(
				vec![
					SlotKey::BottomHand(Side::Right),
					SlotKey::BottomHand(Side::Right),
				],
				ComboSkillDescriptor {
					skill: _Skill,
					..default()
				},
			),
		]]);

		assert_eq!(
			vec![vec![
				ComboTreeElement::Symbol(Symbol::Root),
				ComboTreeElement::Node {
					key_path: vec![SlotKey::BottomHand(Side::Right)],
					descriptor: ComboSkillDescriptor {
						skill: _Skill,
						..default()
					}
				},
				ComboTreeElement::Leaf {
					key_path: vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right)
					],
					descriptor: ComboSkillDescriptor {
						skill: _Skill,
						..default()
					}
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
			vec![(
				vec![SlotKey::BottomHand(Side::Right)],
				ComboSkillDescriptor {
					skill: _Skill,
					..default()
				},
			)],
			vec![(
				vec![SlotKey::BottomHand(Side::Left)],
				ComboSkillDescriptor {
					skill: _Skill,
					..default()
				},
			)],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					}
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![(
				vec![SlotKey::BottomHand(Side::Left)],
				ComboSkillDescriptor {
					skill: _Skill,
					..default()
				},
			)],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![(
				vec![SlotKey::BottomHand(Side::Left)],
				ComboSkillDescriptor {
					skill: _Skill,
					..default()
				},
			)],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![
						SlotKey::BottomHand(Side::Right),
						SlotKey::BottomHand(Side::Left),
					],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right)
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Node {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Leaf {
						key_path: vec![
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Right),
							SlotKey::BottomHand(Side::Left),
							SlotKey::BottomHand(Side::Right),
						],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
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
				(
					vec![SlotKey::BottomHand(Side::Left)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![SlotKey::BottomHand(Side::Left)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Left)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
			vec![
				(
					vec![SlotKey::BottomHand(Side::Right)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
				(
					vec![SlotKey::BottomHand(Side::Left)],
					ComboSkillDescriptor {
						skill: _Skill,
						..default()
					},
				),
			],
		]);

		assert_eq!(
			vec![
				vec![
					ComboTreeElement::Symbol(Symbol::Root),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Line),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Node {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Right)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
				],
				vec![
					ComboTreeElement::Symbol(Symbol::Empty),
					ComboTreeElement::Symbol(Symbol::Corner),
					ComboTreeElement::Leaf {
						key_path: vec![SlotKey::BottomHand(Side::Left)],
						descriptor: ComboSkillDescriptor {
							skill: _Skill,
							..default()
						}
					},
				],
			],
			combos.build_combo_tree_layout()
		);
	}
}
