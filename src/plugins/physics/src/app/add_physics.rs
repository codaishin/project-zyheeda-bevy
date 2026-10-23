use std::any::type_name;

use crate::{
	PhysicsSystems,
	components::ongoing_effects::OngoingEffects,
	observers::skill_prefab::GetCaster,
	systems::interactions::act_on::ActOnSystem,
	traits::act_on::ActOn,
};
use bevy::{ecs::component::Mutable, prelude::*};
use common::prelude::*;

pub(crate) trait AddPhysics {
	fn add_physics<TGetCaster, TActor, TTarget, TSaveGame>(&mut self) -> &mut Self
	where
		TGetCaster: Component + GetCaster,
		TActor: Component<Mutability = Mutable> + SavableComponent + ActOn<TTarget>,
		TTarget: Component<Mutability = Mutable> + SavableComponent,
		OngoingEffects<TActor, TTarget>: Component<Mutability = Mutable> + SavableComponent,
		TSaveGame: HandlesSaving;
}

impl AddPhysics for App {
	fn add_physics<TGetCaster, TActor, TTarget, TSaveGame>(&mut self) -> &mut Self
	where
		TGetCaster: Component + GetCaster,
		TActor: Component<Mutability = Mutable> + SavableComponent + ActOn<TTarget>,
		TTarget: Component<Mutability = Mutable> + SavableComponent,
		OngoingEffects<TActor, TTarget>: Component<Mutability = Mutable> + SavableComponent,
		TSaveGame: HandlesSaving,
	{
		TSaveGame::register_savable_component::<TActor>(self);
		TSaveGame::register_savable_component::<TTarget>(self);
		TSaveGame::register_savable_component::<OngoingEffects<TActor, TTarget>>(self);

		let err =
			self.try_register_required_components::<TActor, OngoingEffects<TActor, TTarget>>();
		if let Err(err) = err {
			error!("{}: {}", type_name::<TActor>(), err);
		}

		self.add_systems(
			FixedPostUpdate,
			FixedPostUpdate::delta
				.pipe(TActor::act_on::<TGetCaster, TTarget>)
				.in_set(PhysicsSystems::Resolve),
		)
	}
}
