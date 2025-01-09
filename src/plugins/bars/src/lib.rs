pub mod components;
mod systems;
mod traits;

use bevy::{
	app::{App, Plugin, Update},
	prelude::IntoSystemConfigs,
	render::{camera::Camera, view::RenderLayers},
};
use common::{
	attributes::health::Health,
	components::UiNodeFor,
	labels::Labels,
	systems::insert_associated::{Configure, InsertAssociated, InsertOn},
	traits::{
		accessors::get::GetterRef,
		handles_enemies::HandlesEnemies,
		handles_graphics::{MainCamera, UiRenderLayer},
		handles_life::HandlesLife,
		handles_player::HandlesPlayer,
		ownership_relation::OwnershipRelation,
		thread_safe::ThreadSafe,
	},
};
use components::Bar;
use std::marker::PhantomData;
use systems::{bar::bar, render_bar::render_bar};

pub struct BarsPlugin<TLifeCycle, TPlayers, TEnemies, TGraphics>(
	PhantomData<(TLifeCycle, TPlayers, TEnemies, TGraphics)>,
);

impl<TLifeCycle, TPlayers, TEnemies, TGraphics>
	BarsPlugin<TLifeCycle, TPlayers, TEnemies, TGraphics>
{
	pub fn depends_on(_: &TLifeCycle, _: &TPlayers, _: &TEnemies, _: &TGraphics) -> Self {
		Self(PhantomData)
	}
}

impl<TLifeCycle, TPlayers, TEnemies, TGraphics> Plugin
	for BarsPlugin<TLifeCycle, TPlayers, TEnemies, TGraphics>
where
	TLifeCycle: ThreadSafe + HandlesLife,
	TPlayers: ThreadSafe + HandlesPlayer,
	TEnemies: ThreadSafe + HandlesEnemies,
	TGraphics: ThreadSafe + UiRenderLayer + MainCamera,
{
	fn build(&self, app: &mut App) {
		let get_health = TLifeCycle::TLife::get;
		let update_life_bars =
			bar::<TLifeCycle::TLife, Health, Camera, TGraphics::TMainCamera>(get_health);
		let render_life_bars = render_bar::<Health>;
		let config_render_layer = Configure::Apply(UiNodeFor::<Bar>::set_render_layer::<TGraphics>);

		app.manage_ownership::<Bar>(Update)
			.add_systems(
				Labels::PREFAB_INSTANTIATION.label(),
				(
					InsertOn::<TPlayers::TPlayer>::associated::<Bar>(Configure::LeaveAsIs),
					InsertOn::<TEnemies::TEnemy>::associated::<Bar>(Configure::LeaveAsIs),
					InsertOn::<UiNodeFor<Bar>>::associated::<RenderLayers>(config_render_layer),
				),
			)
			.add_systems(Update, (update_life_bars, render_life_bars).chain());
	}
}
