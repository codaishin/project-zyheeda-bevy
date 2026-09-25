use super::*;
use crate::assets::agent_meta::Loadout;
use std::{collections::HashSet, iter::Enumerate, slice::Iter};

impl<TContext> ApplyMetaToContext<TContext> for NotLoadedOut
where
	TContext: InsertDefaultLoadout,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta, _: &Agent) {
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
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta, _: &Agent) {
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
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta, _: &Agent) {
		ctx.initialize(meta.bones.skill_mounts.clone(), meta.self_skill_scale);
	}
}

impl<TContext> ApplyMetaToContext<TContext> for NoDefaultAttributes
where
	TContext: ConfigureDefaultAttributes,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta, _: &Agent) {
		ctx.configure_default_attributes(meta.attributes);
	}
}

impl<TContext> ApplyMetaToContext<TContext> for NoBodyConfigured
where
	TContext: ConfigureBody,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta, _: &Agent) {
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
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta, _: &Agent) {
		ctx.configure(meta.speed.with_fastest_left(), meta.required_clearance);
	}
}

impl<TContext> ApplyMetaToContext<TContext> for HasNoRole
where
	TContext: SetRole,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta, agent: &Agent) {
		match agent.agent_type {
			AgentType::Player => {
				let view_offset = meta
					.height_levels
					.view
					.map(|view_height| view_height - *meta.required_clearance.vertical)
					.map(Units::from)
					.unwrap_or_default();
				ctx.set_role(Role::Player { view_offset });
			}
			AgentType::Enemy(_) => {
				ctx.set_role(Role::Enemy);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::assets::agent_meta::HeightLevels;
	use macros::simple_mock;
	use mockall::predicate::eq;
	use testing::Mock;

	mod role {
		use super::*;
		use test_case::test_case;

		simple_mock! {
			_Context {}
			impl SetRole for _Context {
				fn set_role(&mut self, role: Role);
			}
		}

		#[test_case(Some(1.0), Units::ZERO, Units::from(1.); "abs 1")]
		#[test_case(Some(2.0), Units::from(1.), Units::from(1.); "rel 1")]
		#[test_case(None, Units::from(1.), Units::ZERO; "zero if not configured")]
		fn set_player_role_height(view: Option<f32>, vertical: Units, offset: Units) {
			let meta = AgentMeta {
				height_levels: HeightLevels { view, ..default() },
				required_clearance: RequiredClearance {
					vertical,
					..default()
				},
				..default()
			};
			let agent = Agent {
				agent_type: AgentType::Player,
			};
			let mut ctx = Mock_Context::new_mock(assert_player_view_offset(offset));

			HasNoRole::apply_meta_to_context(&mut ctx, &meta, &agent);

			fn assert_player_view_offset(view_offset: Units) -> impl FnMut(&mut Mock_Context) {
				move |mock| {
					mock.expect_set_role()
						.once()
						.with(eq(Role::Player { view_offset }))
						.return_const(());
				}
			}
		}
	}
}
