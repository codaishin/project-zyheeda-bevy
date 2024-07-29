use super::{
	skill_descriptor::SkillDescriptor,
	tooltip::Tooltip,
	DeleteSkill,
	EmptySkillKeySelectDropdownCommand,
	SkillSelectDropdownCommand,
};
use crate::traits::{
	colors::DEFAULT_PANEL_COLORS,
	get_node::GetNode,
	instantiate_content_on::InstantiateContentOn,
	CombosDescriptor,
	UpdateCombos,
};
use bevy::{
	asset::Handle,
	color::Color,
	hierarchy::{BuildChildren, ChildBuilder},
	prelude::{Bundle, Component, KeyCode},
	render::texture::Image,
	text::TextStyle,
	ui::{
		node_bundles::{ButtonBundle, NodeBundle, TextBundle},
		AlignItems,
		FlexDirection,
		JustifyContent,
		PositionType,
		Style,
		UiImage,
		UiRect,
		Val,
	},
	utils::default,
};
use common::traits::get_ui_text::{English, GetUiText, UIText};

#[derive(Component, Default)]
pub(crate) struct ComboOverview(CombosDescriptor<KeyCode>);

pub(crate) trait SkillButtonBundle {
	fn with<TKey>(self, descriptor: &SkillDescriptor<TKey>) -> impl Bundle
	where
		TKey: Clone + Sync + Send + 'static;
}

impl SkillButtonBundle for ButtonBundle {
	fn with<TKey>(self, descriptor: &SkillDescriptor<TKey>) -> impl Bundle
	where
		TKey: Clone + Sync + Send + 'static,
	{
		(
			self,
			descriptor.clone(),
			Tooltip::new(descriptor.skill.clone()),
		)
	}
}

impl ComboOverview {
	pub(crate) fn skill_container_bundle() -> impl Bundle {
		NodeBundle {
			style: Style {
				margin: UiRect::all(Val::Px(5.0)),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn skill_button_bundle(
		icon: Option<Handle<Image>>,
	) -> impl SkillButtonBundle + Bundle {
		ButtonBundle {
			style: Style {
				width: Val::Px(65.0),
				height: Val::Px(65.0),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			background_color: DEFAULT_PANEL_COLORS.filled.into(),
			image: icon.map(UiImage::new).unwrap_or_default(),
			..default()
		}
	}

	pub(crate) fn skill_key_button_offset_container() -> impl Bundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				top: Val::Px(-8.0),
				right: Val::Px(-8.0),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn delete_button_offset_container() -> impl Bundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				bottom: Val::Px(-4.0),
				right: Val::Px(-4.0),
				..default()
			},
			..default()
		}
	}

	pub(crate) fn skill_key_button_bundle() -> impl Bundle {
		ButtonBundle {
			style: Style {
				width: Val::Px(50.0),
				height: Val::Px(25.0),
				border: UiRect::all(Val::Px(2.0)),
				margin: UiRect::all(Val::Px(-2.0)),
				justify_content: JustifyContent::Center,
				..default()
			},
			background_color: DEFAULT_PANEL_COLORS.filled.into(),
			border_color: DEFAULT_PANEL_COLORS.text.into(),
			..default()
		}
	}

	pub(crate) fn delete_button_bundle() -> impl Bundle {
		ButtonBundle {
			style: Style {
				width: Val::Px(20.0),
				height: Val::Px(20.0),
				border: UiRect::all(Val::Px(2.0)),
				margin: UiRect::all(Val::Px(-2.0)),
				justify_content: JustifyContent::Center,
				..default()
			},
			background_color: DEFAULT_PANEL_COLORS.filled.into(),
			border_color: DEFAULT_PANEL_COLORS.text.into(),
			..default()
		}
	}

	pub(crate) fn skill_key_text(key: &str) -> impl Bundle {
		TextBundle::from_section(
			key,
			TextStyle {
				font_size: 20.,
				color: DEFAULT_PANEL_COLORS.text,
				..default()
			},
		)
	}

	pub(crate) fn new_skill_text(key: &str) -> impl Bundle {
		TextBundle::from_section(
			key,
			TextStyle {
				font_size: 50.,
				color: DEFAULT_PANEL_COLORS.text,
				..default()
			},
		)
	}

	pub(crate) fn delete_button_text(key: &str) -> impl Bundle {
		TextBundle::from_section(
			key,
			TextStyle {
				font_size: 15.,
				color: DEFAULT_PANEL_COLORS.text,
				..default()
			},
		)
	}
}

impl UpdateCombos<KeyCode> for ComboOverview {
	fn update_combos(&mut self, combos: CombosDescriptor<KeyCode>) {
		self.0 = combos
	}
}

impl GetNode for ComboOverview {
	fn node(&self) -> NodeBundle {
		NodeBundle {
			style: Style {
				position_type: PositionType::Absolute,
				top: Val::Px(30.0),
				left: Val::Px(30.0),
				right: Val::Px(30.0),
				bottom: Val::Px(30.0),
				flex_direction: FlexDirection::Column,
				..default()
			},
			background_color: Color::srgba(0.5, 0.5, 0.5, 0.5).into(),
			..default()
		}
	}
}

impl InstantiateContentOn for ComboOverview {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		add_title(parent, "Combos");
		add_combo_list(parent, self);
	}
}

fn add_title(parent: &mut ChildBuilder, title: &str) {
	parent
		.spawn(NodeBundle {
			style: Style {
				margin: UiRect::all(Val::Px(10.)),
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			parent.spawn(TextBundle::from_section(
				title,
				TextStyle {
					font_size: 40.0,
					color: DEFAULT_PANEL_COLORS.text,
					..default()
				},
			));
		});
}

fn add_combo_list(parent: &mut ChildBuilder, combo_overview: &ComboOverview) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Column,
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			for combo in &combo_overview.0 {
				add_combo(parent, combo);
			}
		});
}

fn add_combo(parent: &mut ChildBuilder, combo: &[SkillDescriptor<KeyCode>]) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Row,
				margin: UiRect::top(Val::Px(10.0)),
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			let last = combo.len() - 1;

			for skill in &combo[..last] {
				add_skill(parent, skill, &[with_key_button]);
			}

			let Some(last_skill) = combo.last() else {
				return;
			};

			add_skill(parent, last_skill, &[with_key_button, with_delete_button]);
			add_empty_skill(parent, last_skill.key_path.clone());
		});
}

fn add_skill(
	parent: &mut ChildBuilder,
	descriptor: &SkillDescriptor<KeyCode>,
	additional_buttons: &[fn(&SkillDescriptor<KeyCode>, &mut ChildBuilder)],
) {
	let skill_icon = descriptor.skill.icon.clone();

	parent
		.spawn(ComboOverview::skill_container_bundle())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::skill_button_bundle(skill_icon).with(descriptor),
					SkillSelectDropdownCommand {
						key_path: descriptor.key_path.clone(),
					},
				))
				.with_children(|parent| {
					for additional_button in additional_buttons {
						additional_button(descriptor, parent);
					}
				});
		});
}

fn with_key_button(descriptor: &SkillDescriptor<KeyCode>, parent: &mut ChildBuilder) {
	let skill_key = match descriptor.key_path.last().map(English::ui_text) {
		Some(UIText::String(key)) => key,
		None | Some(UIText::Unmapped) => String::from("?"),
	};

	parent
		.spawn(ComboOverview::skill_key_button_offset_container())
		.with_children(|parent| {
			parent
				.spawn(ComboOverview::skill_key_button_bundle())
				.with_children(|parent| {
					parent.spawn(ComboOverview::skill_key_text(&skill_key));
				});
		});
}

fn with_delete_button(descriptor: &SkillDescriptor<KeyCode>, parent: &mut ChildBuilder) {
	parent
		.spawn(ComboOverview::delete_button_offset_container())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::delete_button_bundle(),
					DeleteSkill {
						key_path: descriptor.key_path.clone(),
					},
				))
				.with_children(|parent| {
					parent.spawn(ComboOverview::delete_button_text("X"));
				});
		});
}

fn add_empty_skill(parent: &mut ChildBuilder, key_path: Vec<KeyCode>) {
	parent
		.spawn(ComboOverview::skill_container_bundle())
		.with_children(|parent| {
			parent
				.spawn(ComboOverview::skill_button_bundle(None))
				.with_children(|parent| {
					let target = parent.parent_entity();

					parent.spawn(ComboOverview::new_skill_text(""));
					parent
						.spawn(ComboOverview::skill_key_button_offset_container())
						.with_children(|parent| {
							parent
								.spawn((
									ComboOverview::skill_key_button_bundle(),
									EmptySkillKeySelectDropdownCommand { target, key_path },
								))
								.with_children(|parent| {
									parent.spawn(ComboOverview::skill_key_text("+"));
								});
						});
				});
		});
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::asset::AssetId;
	use skills::skills::Skill;
	use uuid::Uuid;

	#[test]
	fn update_combos() {
		let combos = vec![vec![SkillDescriptor {
			key_path: vec![KeyCode::ArrowLeft],
			skill: Skill {
				name: "my skill".to_owned(),
				icon: Some(Handle::Weak(AssetId::Uuid {
					uuid: Uuid::new_v4(),
				})),
				..default()
			},
		}]];
		let mut combo_overview = ComboOverview::default();
		combo_overview.update_combos(combos.clone());

		assert_eq!(combos, combo_overview.0)
	}
}
