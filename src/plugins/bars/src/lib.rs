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
	components::UiNodeFor,
	traits::{
		accessors::get::GetterRef,
		handles_enemies::HandlesEnemies,
		handles_graphics::UiCamera,
		handles_life::HandlesLife,
		handles_player::HandlesPlayer,
		ownership_relation::OwnershipRelation,
		register_required_components_mapped::RegisterRequiredComponentsMapped,
		thread_safe::ThreadSafe,
	},
};
use components::bar::Bar;
use std::marker::PhantomData;
use systems::{bar::bar, render_bar::render_bar};

pub struct BarsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLifeCycle, TPlayers, TEnemies, TGraphics>
	BarsPlugin<(TLifeCycle, TPlayers, TEnemies, TGraphics)>
where
	TLifeCycle: ThreadSafe + HandlesLife,
	TPlayers: ThreadSafe + HandlesPlayer,
	TEnemies: ThreadSafe + HandlesEnemies,
	TGraphics: ThreadSafe + UiCamera,
{
	pub fn from_plugins(_: &TLifeCycle, _: &TPlayers, _: &TEnemies, _: &TGraphics) -> Self {
		Self(PhantomData)
	}
}

impl<TLifeCycle, TPlayers, TEnemies, TGraphics> Plugin
	for BarsPlugin<(TLifeCycle, TPlayers, TEnemies, TGraphics)>
where
	TLifeCycle: ThreadSafe + HandlesLife,
	TPlayers: ThreadSafe + HandlesPlayer,
	TEnemies: ThreadSafe + HandlesEnemies,
	TGraphics: ThreadSafe + UiCamera,
{
	fn build(&self, app: &mut App) {
		let get_health = TLifeCycle::TLife::get;
		let update_life_bars =
			bar::<TLifeCycle::TLife, Health, Camera, TGraphics::TUiCamera>(get_health);
		let render_life_bars = render_bar::<Health>;
		let render_layer = UiNodeFor::<Bar>::render_layer::<TGraphics::TUiCamera>;

		app.register_required_components::<TPlayers::TPlayer, Bar>()
			.register_required_components::<TEnemies::TEnemy, Bar>()
			.register_required_components_mapped::<UiNodeFor<Bar>, RenderLayers>(render_layer);
		app.manage_ownership::<Bar>(Update)
			.add_systems(Update, (update_life_bars, render_life_bars).chain());
	}
}
