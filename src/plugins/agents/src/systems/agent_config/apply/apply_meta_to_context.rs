use super::*;
use crate::assets::agent_meta::Loadout;
use std::{collections::HashSet, iter::Enumerate, slice::Iter};

impl<TContext> ApplyMetaToContext<TContext> for NotLoadedOut
where
	TContext: InsertDefaultLoadout,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta) {
		ctx.insert_default_loadout(&meta.loadout);
	}
}

pub struct LoadoutIterator<'a> {
	inventory: Enumerate<Iter<'a, Option<ItemName>>>,
	slots: Iter<'a, (SlotKey, Option<ItemName>)>,
}

impl LoadoutIterator<'_> {
	fn next_inventory_item(&mut self) -> Option<(LoadoutKey, Option<ItemName>)> {
		self.inventory
			.next()
			.map(|(key, item)| (LoadoutKey::from(InventoryKey(key)), item.clone()))
	}

	fn next_slot_item(&mut self) -> Option<(LoadoutKey, Option<ItemName>)> {
		self.slots
			.next()
			.map(|(key, item)| (LoadoutKey::from(*key), item.clone()))
	}
}

impl Iterator for LoadoutIterator<'_> {
	type Item = (LoadoutKey, Option<ItemName>);

	fn next(&mut self) -> Option<Self::Item> {
		self.next_inventory_item().or_else(|| self.next_slot_item())
	}
}

impl<'a> IntoIterator for &'a Loadout {
	type Item = (LoadoutKey, Option<ItemName>);
	type IntoIter = LoadoutIterator<'a>;

	fn into_iter(self) -> LoadoutIterator<'a> {
		LoadoutIterator {
			inventory: self.inventory.iter().enumerate(),
			slots: self.slots.iter(),
		}
	}
}

impl<TContext> ApplyMetaToContext<TContext> for NoBonesRegistered
where
	TContext: RegisterLoadoutBones,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta) {
		ctx.register_loadout_bones(
			meta.bones.forearm_slots.clone(),
			meta.bones.hand_slots.clone(),
			meta.bones.essence_slots.clone(),
		);
	}
}

impl<TContext> ApplyMetaToContext<TContext> for NotInitializedAgent
where
	TContext: Initialize,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta) {
		ctx.initialize(meta.bones.skill_mounts.clone(), meta.self_skill_scale);
	}
}

impl<TContext> ApplyMetaToContext<TContext> for NoDefaultAttributes
where
	TContext: ConfigureDefaultAttributes,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta) {
		ctx.configure_default_attributes(meta.attributes);
	}
}

impl<TContext> ApplyMetaToContext<TContext> for NoBodyConfigured
where
	TContext: ConfigureBody,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta) {
		let half_y =
			Units::from(*meta.required_clearance.vertical - *meta.required_clearance.horizontal);
		let radius = meta.required_clearance.horizontal;
		let center = meta.height_levels.center - *meta.required_clearance.vertical;
		let aim = meta.height_levels.aim - *meta.required_clearance.vertical;
		ctx.configure_body(
			BodyConfig {
				core: Some(Core {
					shape: Shape::Parameters(ShapeParameters::Capsule { half_y, radius }),
					physics_type: PhysicsType::Agent(HashSet::from([Blocker::Character])),
				}),
				sub_frames: vec![meta.interactive_detection_shape],
			},
			TranslationOffsets { center, aim },
		);
	}
}

impl<TContext> ApplyMetaToContext<TContext> for NotConfiguredMovement
where
	TContext: ConfigureMovement,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta) {
		ctx.configure(meta.speed.with_fastest_left(), meta.required_clearance);
	}
}
