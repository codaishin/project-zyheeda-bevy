use super::{tooltip::Tooltip, EmptySkillKeySelectDropdownCommand, SkillSelectDropdownCommand};
use crate::{
	tools::SkillDescriptor,
	traits::{
		colors::DEFAULT_PANEL_COLORS,
		get_node::GetNode,
		instantiate_content_on::InstantiateContentOn,
		CombosDescriptor,
		UpdateCombos,
	},
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
pub(crate) struct ComboOverview(CombosDescriptor<KeyCode, Handle<Image>>);

impl ComboOverview {
	pub fn skill_container_bundle() -> impl Bundle {
		NodeBundle {
			style: Style {
				margin: UiRect::all(Val::Px(5.0)),
				..default()
			},
			..default()
		}
	}

	pub fn skill_button_bundle(icon: Option<Handle<Image>>) -> impl Bundle {
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
	pub fn skill_key_button_offset_container() -> impl Bundle {
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

	pub fn skill_key_button_bundle() -> impl Bundle {
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

	pub fn skill_key_text(key: &str) -> impl Bundle {
		TextBundle::from_section(
			key,
			TextStyle {
				font_size: 20.,
				color: DEFAULT_PANEL_COLORS.text,
				..default()
			},
		)
	}

	pub fn new_skill_text(key: &str) -> impl Bundle {
		TextBundle::from_section(
			key,
			TextStyle {
				font_size: 50.,
				color: DEFAULT_PANEL_COLORS.text,
				..default()
			},
		)
	}
}

impl UpdateCombos<KeyCode> for ComboOverview {
	fn update_combos(&mut self, combos: CombosDescriptor<KeyCode, Handle<Image>>) {
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

fn add_combo(parent: &mut ChildBuilder, combo: &Vec<SkillDescriptor<KeyCode, Handle<Image>>>) {
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
			for skill in combo {
				add_skill(parent, skill);
			}
			let Some(skill) = combo.last() else {
				return;
			};
			add_empty_skill(parent, skill.key_path.clone());
		});
}

fn add_skill(parent: &mut ChildBuilder, skill: &SkillDescriptor<KeyCode, Handle<Image>>) {
	let skill_key = match skill.key_path.last().map(English::ui_text) {
		Some(UIText::String(key)) => key,
		None | Some(UIText::Unmapped) => String::from("?"),
	};
	let skill_icon = skill.icon.clone().unwrap_or_default();

	parent
		.spawn(ComboOverview::skill_container_bundle())
		.with_children(|parent| {
			parent
				.spawn((
					ComboOverview::skill_button_bundle(Some(skill_icon)),
					Tooltip(skill.clone()),
					SkillSelectDropdownCommand {
						key_path: skill.key_path.clone(),
					},
				))
				.with_children(|parent| {
					parent
						.spawn(ComboOverview::skill_key_button_offset_container())
						.with_children(|parent| {
							parent
								.spawn(ComboOverview::skill_key_button_bundle())
								.with_children(|parent| {
									parent.spawn(ComboOverview::skill_key_text(&skill_key));
								});
						});
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
	use uuid::Uuid;

	#[test]
	fn update_combos() {
		let combos = vec![vec![SkillDescriptor {
			name: "my skill".to_owned(),
			key_path: vec![KeyCode::ArrowLeft],
			icon: Some(Handle::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			})),
		}]];
		let mut combo_overview = ComboOverview::default();
		combo_overview.update_combos(combos.clone());

		assert_eq!(combos, combo_overview.0)
	}
}
