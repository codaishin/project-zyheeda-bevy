pub(crate) mod related;

use bevy::prelude::Entity;

pub(crate) fn get_recursively_from<'a, TRelatedEntities, TRelated>(
	root_entity: Entity,
	fetch_related: &'a impl Fn(Entity) -> Option<TRelatedEntities>,
	repeat_for_related: &'a impl Fn(&TRelated) -> bool,
) -> Box<dyn Iterator<Item = Entity> + 'a>
where
	TRelatedEntities: Iterator<Item = TRelated> + 'a,
	TRelated: 'a,
	Entity: From<TRelated>,
{
	Box::new(
		Iter(fetch_related(root_entity))
			.filter(repeat_for_related)
			.map(Entity::from)
			.flat_map(|entity| get_recursively_from(entity, fetch_related, repeat_for_related))
			.chain([root_entity]),
	)
}

struct Iter<TRelatedEntities, TRelated>(Option<TRelatedEntities>)
where
	TRelatedEntities: Iterator<Item = TRelated>;

impl<TRelatedEntities, TRelated> Iterator for Iter<TRelatedEntities, TRelated>
where
	TRelatedEntities: Iterator<Item = TRelated>,
{
	type Item = TRelated;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.0 {
			None => None,
			Some(it) => it.next(),
		}
	}
}
