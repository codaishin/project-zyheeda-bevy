use super::view::ItemView;

pub trait GetViewData<TView, TKey>
where
	TView: ItemView<TKey>,
{
	fn get_view_data(&self) -> TView::TViewComponents;
}
