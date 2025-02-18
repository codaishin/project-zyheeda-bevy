pub trait InspectAble<TField>
where
	TField: InspectMarker,
{
	fn get_inspect_able_field(&self) -> TField::TFieldRef<'_>;
}

pub trait InspectField<TSource>: InspectMarker {
	fn inspect_field(source: &TSource) -> Self::TFieldRef<'_>;
}

pub trait InspectMarker {
	type TFieldRef<'a>;
}

impl<TSource, T> InspectField<TSource> for T
where
	T: InspectMarker,
	TSource: InspectAble<T>,
{
	fn inspect_field(source: &TSource) -> Self::TFieldRef<'_> {
		source.get_inspect_able_field()
	}
}
