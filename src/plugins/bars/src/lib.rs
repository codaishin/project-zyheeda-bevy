pub mod components;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Update},
	ecs::schedule::IntoScheduleConfigs,
	render::{camera::Camera, view::RenderLayers},
};
use common::{
	attributes::health::Health,
	components::{life::Life, ui_node_for::UiNodeFor},
	traits::{
		accessors::get::GetterRef,
		handles_enemies::HandlesEnemyBehaviors,
		handles_graphics::UiCamera,
		handles_player::HandlesPlayer,
		ownership_relation::OwnershipRelation,
		thread_safe::ThreadSafe,
	},
};
use components::bar::Bar;
use std::marker::PhantomData;
use systems::{bar::bar, render_bar::render_bar};

pub struct BarsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TPlayers, TEnemies, TGraphics> BarsPlugin<(TPlayers, TEnemies, TGraphics)>
where
	TPlayers: ThreadSafe + HandlesPlayer,
	TEnemies: ThreadSafe + HandlesEnemyBehaviors,
	TGraphics: ThreadSafe + UiCamera,
{
	pub fn from_plugins(_: &TPlayers, _: &TEnemies, _: &TGraphics) -> Self {
		Self(PhantomData)
	}
}

impl<TPlayers, TEnemies, TGraphics> Plugin for BarsPlugin<(TPlayers, TEnemies, TGraphics)>
where
	TPlayers: ThreadSafe + HandlesPlayer,
	TEnemies: ThreadSafe + HandlesEnemyBehaviors,
	TGraphics: ThreadSafe + UiCamera,
{
	fn build(&self, app: &mut App) {
		let update_life_bars = bar::<Life, Health, Camera, TGraphics::TUiCamera>(Life::get);
		let render_life_bars = render_bar::<Health>;
		let render_layer = UiNodeFor::<Bar>::render_layer::<TGraphics::TUiCamera>;

		app.register_required_components::<TPlayers::TPlayer, Bar>()
			.register_required_components::<TEnemies::TEnemyBehavior, Bar>()
			.register_required_components_with::<UiNodeFor<Bar>, RenderLayers>(render_layer);
		app.manage_ownership::<Bar>(Update)
			.add_systems(Update, (update_life_bars, render_life_bars).chain());
	}
}
