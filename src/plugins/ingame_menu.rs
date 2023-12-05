use crate::{
	components::{Collection, Inventory, InventoryKey, Player, Side, Slot, SlotKey, Slots, Swap},
	states::GameState,
};
use bevy::prelude::*;
use std::marker::PhantomData;

const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
const EMPTY_BUTTON: Color = Color::rgb(0.35, 0.35, 0.35);
const BUSY_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const BACKGROUND_COLOR: Color = Color::rgba(0.5, 0.5, 0.5, 0.5);
const EQUIPMENT_SLOTS: [(SlotKey, &str); 2] = [
	(SlotKey::Hand(Side::Left), "Left Hand"),
	(SlotKey::Hand(Side::Right), "Right Hand"),
];

pub struct IngameMenuPlugin;

#[derive(Component, Default)]
pub struct InventoryScreen;

#[derive(Component)]
pub struct Busy;

#[derive(Component, Debug)]
struct Panel<T> {
	pub index: usize,
	phantom_data: PhantomData<T>,
}

impl<T> Panel<T> {
	pub fn new(index: usize) -> Self {
		Self {
			index,
			phantom_data: PhantomData,
		}
	}
}

#[derive(Component, Debug)]
struct Drag<T> {
	pub value: T,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum MenuState {
	#[default]
	None,
	Inventory,
}

impl Plugin for IngameMenuPlugin {
	fn build(&self, app: &mut App) {
		app.add_state::<MenuState>()
			.add_systems(Update, toggle_inventory)
			.add_systems(
				OnEnter(MenuState::Inventory),
				(spawn_screen::<InventoryScreen>, run_game::<false>),
			)
			.add_systems(
				OnExit(MenuState::Inventory),
				(despawn_screen::<InventoryScreen>, run_game::<true>),
			)
			.add_systems(
				Update,
				(color_buttons, drag_and_drop, update_slots, update_item)
					.run_if(in_state(MenuState::Inventory)),
			);
	}
}

fn run_game<const RUN: bool>(mut next_state: ResMut<NextState<GameState>>) {
	if RUN {
		next_state.set(GameState::Running);
	} else {
		next_state.set(GameState::InGameMenu);
	}
}

fn toggle_inventory(
	keys: Res<Input<KeyCode>>,
	current_state: Res<State<MenuState>>,
	mut next_state: ResMut<NextState<MenuState>>,
) {
	if keys.just_pressed(KeyCode::I) {
		let state = match current_state.get() {
			MenuState::Inventory => MenuState::None,
			MenuState::None => MenuState::Inventory,
		};
		next_state.set(state);
	}
}

fn despawn_screen<TScreen: Component>(
	mut commands: Commands,
	screens: Query<Entity, With<TScreen>>,
) {
	for entity in &screens {
		commands.entity(entity).despawn_recursive();
	}
}

fn get_panel_button() -> ButtonBundle {
	let slot_style = Style {
		width: Val::Px(65.0),
		height: Val::Px(65.0),
		margin: UiRect::all(Val::Px(2.0)),
		justify_content: JustifyContent::Center,
		align_items: AlignItems::Center,
		..default()
	};
	ButtonBundle {
		style: slot_style.clone(),
		background_color: EMPTY_BUTTON.into(),
		..default()
	}
}

fn add_title(parent: &mut ChildBuilder, title: &str) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Center,
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			parent.spawn(TextBundle::from_section(
				title,
				TextStyle {
					font_size: 40.0,
					color: TEXT_COLOR,
					..default()
				},
			));
		});
}

fn add<T: Sync + Send + 'static>(
	parent: &mut ChildBuilder,
	label: Option<&str>,
	x: u32,
	y: u32,
	start_index: usize,
) {
	let mut index = start_index;
	for _ in 0..y {
		parent
			.spawn(NodeBundle {
				style: Style {
					flex_direction: FlexDirection::Row,
					align_items: AlignItems::Center,
					..default()
				},
				..default()
			})
			.with_children(|parent| {
				if let Some(label) = label {
					parent.spawn(TextBundle::from_section(
						label,
						TextStyle {
							font_size: 20.0,
							color: TEXT_COLOR,
							..default()
						},
					));
				}
				for _ in 0..x {
					parent
						.spawn((Panel::<T>::new(index), get_panel_button()))
						.with_children(|parent| {
							parent.spawn(TextBundle::from_section(
								"<Empty>",
								TextStyle {
									font_size: 15.0,
									color: TEXT_COLOR,
									..default()
								},
							));
						});
					index += 1;
				}
			});
	}
}

fn add_inventory(parent: &mut ChildBuilder) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
				margin: UiRect::all(Val::Px(5.0)),
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			add_title(parent, "Inventory");
			add::<Inventory>(parent, None, 5, 5, 0);
		});
}

fn add_equipment(parent: &mut ChildBuilder) {
	parent
		.spawn(NodeBundle {
			style: Style {
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::End,
				margin: UiRect::all(Val::Px(5.0)),
				..default()
			},
			..default()
		})
		.with_children(|parent| {
			add_title(parent, "Equipment");
			for (index, (_, name)) in EQUIPMENT_SLOTS.iter().enumerate() {
				add::<Slot>(parent, Some(name), 1, 1, index);
			}
		});
}

fn spawn_screen<TScreen: Component + Default>(mut commands: Commands) {
	commands
		.spawn((
			NodeBundle {
				style: Style {
					width: Val::Vw(100.0),
					height: Val::Vh(100.0),
					align_items: AlignItems::Center,
					justify_content: JustifyContent::Center,
					..default()
				},
				background_color: BACKGROUND_COLOR.into(),
				..default()
			},
			TScreen::default(),
		))
		.with_children(|parent| {
			parent
				.spawn(NodeBundle {
					style: Style {
						flex_direction: FlexDirection::Row,
						align_items: AlignItems::Start,
						..default()
					},
					..default()
				})
				.with_children(add_equipment)
				.with_children(add_inventory);
		});
}

type ButtonColorComponents<'a> = (&'a Interaction, &'a mut BackgroundColor, Option<&'a Busy>);

fn color_buttons(mut interaction_query: Query<ButtonColorComponents, With<Button>>) {
	for (interaction, mut color, busy) in &mut interaction_query {
		*color = match (interaction, busy) {
			(Interaction::Pressed, _) => PRESSED_BUTTON.into(),
			(Interaction::Hovered, _) => HOVERED_BUTTON.into(),
			(Interaction::None, Some(_)) => BUSY_BUTTON.into(),
			(Interaction::None, None) => EMPTY_BUTTON.into(),
		}
	}
}

fn update_slots(
	mut commands: Commands,
	slots: Query<&Slots>,
	agents: Query<Entity, With<Player>>,
	mut texts: Query<&mut Text>,
	mut slot_buttons: Query<(Entity, &Panel<Slot>, &Children), With<Button>>,
) {
	let player = agents.single();

	for (entity, panel, children) in &mut slot_buttons {
		let mut entity = commands.entity(entity);
		let mut txt = texts.get_mut(children[0]).unwrap();
		let slots = slots.get(player).unwrap();
		let (slot_key, _) = EQUIPMENT_SLOTS[panel.index];
		match slots.0.get(&slot_key).and_then(|s| s.item) {
			Some(item) => {
				txt.sections[0].value = item.name.to_string();
				entity.insert(Busy);
			}
			_ => {
				txt.sections[0].value = "<Empty>".to_string();
				entity.remove::<Busy>();
			}
		};
	}
}

fn update_item(
	mut commands: Commands,
	inventory: Query<&Inventory>,
	agents: Query<Entity, With<Player>>,
	mut texts: Query<&mut Text>,
	inventory_buttons: Query<(Entity, &Panel<Inventory>, &Children), With<Button>>,
) {
	let player = agents.single();

	for (entity, panel, children) in &inventory_buttons {
		let mut entity = commands.entity(entity);
		let mut txt = texts.get_mut(children[0]).unwrap();
		let inventory = inventory.get(player).unwrap();
		match inventory.0.get(panel.index) {
			Some(Some(item)) => {
				txt.sections[0].value = item.name.to_string();
				entity.insert(Busy);
			}
			_ => {
				txt.sections[0].value = "<Empty>".to_string();
				entity.remove::<Busy>();
			}
		};
	}
}

type Panels<'a> = (
	&'a Interaction,
	Option<&'a Panel<Inventory>>,
	Option<&'a Panel<Slot>>,
);

fn drag_and_drop(
	mut commands: Commands,
	mouse: Res<Input<MouseButton>>,
	agents: Query<Entity, With<Player>>,
	drag_from_inventory: Query<&Drag<usize>>,
	drag_from_equipment: Query<&Drag<SlotKey>>,
	mut panels: Query<Panels, With<Button>>,
) {
	let player = agents.single();

	for (interaction, inv_p, equ_p) in &mut panels {
		let mouse_pressed = mouse.just_pressed(MouseButton::Left);
		let mouse_released = mouse.just_released(MouseButton::Left);
		let inv_d = drag_from_inventory.get(player);
		let equ_d = drag_from_equipment.get(player);
		match (interaction, mouse_pressed, mouse_released) {
			(Interaction::Pressed, true, false) => match (inv_p, equ_p) {
				(Some(panel), _) => {
					let value = panel.index;
					commands.entity(player).insert(Drag { value });
				}
				(_, Some(panel)) => {
					let (value, _) = EQUIPMENT_SLOTS[panel.index];
					commands.entity(player).insert(Drag { value });
				}
				_ => {
					println!("PRESSING INCOMPATIBLE BUTTONS, SHOULD NOT HAVE HAPPENED")
				}
			},
			(Interaction::Hovered, false, true) => match (inv_p, inv_d, equ_p, equ_d) {
				(Some(inv_p), _, _, Ok(equ_d)) => {
					let inventory_key = inv_p.index;
					let slot_key = equ_d.value;
					commands.entity(player).insert(Collection::new([Swap(
						InventoryKey(inventory_key),
						slot_key,
					)]));
					commands.entity(player).remove::<Drag<SlotKey>>();
				}
				(_, _, Some(equ_p), Ok(equ_d)) => {
					let (slot_key, _) = EQUIPMENT_SLOTS[equ_p.index];
					commands
						.entity(player)
						.insert(Collection::new([Swap(slot_key, equ_d.value)]));
					commands.entity(player).remove::<Drag<SlotKey>>();
				}
				(_, Ok(inv_d), Some(equ_p), _) => {
					let (slot_key, _) = EQUIPMENT_SLOTS[equ_p.index];
					let inventory_key = inv_d.value;
					commands.entity(player).insert(Collection::new([Swap(
						InventoryKey(inventory_key),
						slot_key,
					)]));
					commands.entity(player).remove::<Drag<usize>>();
				}
				(Some(inv_p), Ok(inv_d), ..) => {
					commands.entity(player).insert(Collection::new([Swap(
						InventoryKey(inv_p.index),
						InventoryKey(inv_d.value),
					)]));
					commands.entity(player).remove::<Drag<usize>>();
				}
				combination => {
					println!(
						"DRAGGING INCOMPATIBLE BUTTONS {:?} (SHOULD NOT HAPPEN)",
						combination
					);
				}
			},
			_ => {}
		}
	}
}
