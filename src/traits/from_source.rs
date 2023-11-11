pub mod behavior;

pub trait FromSource<TSource, TData>
where
	Self: Sized,
{
	fn from_source(source: TSource, data: TData) -> Option<Self>;
}
