use super::tooltip::Tooltip;
use crate::traits::{
	children::Children,
	colors::{HasBackgroundColor, DEFAULT_PANEL_COLORS},
	get_node::GetNode,
	CombosDescriptor,
	SkillDescriptor,
	UpdateCombos,
};
use bevy::{
	asset::Handle,
	hierarchy::{BuildChildren, ChildBuilder},
	prelude::{Component, KeyCode},
	render::{color::Color, texture::Image},
	text::TextStyle,
	ui::{
		node_bundles::{ButtonBundle, NodeBundle, TextBundle},
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
			background_color: Color::rgba(0.5, 0.5, 0.5, 0.5).into(),
			..default()
		}
	}
}

impl HasBackgroundColor for ComboOverview {
	const BACKGROUND_COLOR: Option<Color> = Some(Color::rgba(0.5, 0.5, 0.5, 0.5));
}

impl Children for ComboOverview {
	fn children(&self, parent: &mut ChildBuilder) {
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
		});
}

fn add_skill(parent: &mut ChildBuilder, skill: &SkillDescriptor<KeyCode, Handle<Image>>) {
	let skill_key = match English::ui_text(&skill.key) {
		UIText::String(v) => v,
		_ => String::from("?"),
	};
	parent
		.spawn(NodeBundle {
			style: Style {
				margin: UiRect::all(Val::Px(5.0)),
				..default()
			},
			background_color: DEFAULT_PANEL_COLORS.empty.into(),
			..default()
		})
		.with_children(|parent| {
			parent
				.spawn((
					ButtonBundle {
						style: Style {
							width: Val::Px(65.0),
							height: Val::Px(65.0),
							..default()
						},
						background_color: DEFAULT_PANEL_COLORS.text.into(),
						image: UiImage::new(skill.icon.clone().unwrap_or_default()),
						..default()
					},
					Tooltip(skill.clone()),
				))
				.with_children(|parent| {
					parent
						.spawn(NodeBundle {
							style: Style {
								position_type: PositionType::Absolute,
								width: Val::Px(50.0),
								height: Val::Px(25.0),
								top: Val::Px(-8.0),
								right: Val::Px(-8.0),
								border: UiRect::all(Val::Px(2.0)),
								justify_content: JustifyContent::Center,
								..default()
							},
							background_color: DEFAULT_PANEL_COLORS.empty.into(),
							border_color: DEFAULT_PANEL_COLORS.text.into(),
							..default()
						})
						.with_children(|parent| {
							parent.spawn(TextBundle::from_section(
								skill_key,
								TextStyle {
									font_size: 20.,
									color: DEFAULT_PANEL_COLORS.text,
									..default()
								},
							));
						});
				});
		});
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::SkillDescriptor;
	use bevy::{asset::AssetId, utils::Uuid};

	#[test]
	fn update_combos() {
		let combos = vec![vec![SkillDescriptor {
			name: "my skill",
			key: KeyCode::ArrowLeft,
			icon: Some(Handle::Weak(AssetId::Uuid {
				uuid: Uuid::new_v4(),
			})),
		}]];
		let mut combo_overview = ComboOverview::default();
		combo_overview.update_combos(combos.clone());

		assert_eq!(combos, combo_overview.0)
	}
}
