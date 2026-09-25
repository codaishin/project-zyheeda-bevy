use super::*;

impl<TContext> ApplyMetaToContext<TContext> for NotLoadedOut
where
	TContext: InsertDefaultLoadout,
{
	fn apply_meta_to_context(ctx: &mut TContext, meta: &AgentMeta) {
		ctx.insert_default_loadout(&meta.loadout);
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
