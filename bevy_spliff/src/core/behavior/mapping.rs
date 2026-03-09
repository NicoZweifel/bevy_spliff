use crate::core::prelude::*;
use bevy_ecs::query::QueryData;

impl JoinResultMapper for SingleJoin {
    type Item<'w, 's, D: QueryData> = Option<D::Item<'w, 's>>;

    fn map_results<'w, 's, D: QueryData>(mut items: Vec<D::Item<'w, 's>>) -> Self::Item<'w, 's, D> {
        items.pop()
    }

    fn shrink<'wlong: 'wshort, 'wshort, 's, D: QueryData>(
        item: Self::Item<'wlong, 's, D>,
    ) -> Self::Item<'wshort, 's, D> {
        item.map(D::shrink)
    }
}

impl JoinResultMapper for MultipleJoin {
    type Item<'w, 's, D: QueryData> = Vec<D::Item<'w, 's>>;

    fn map_results<'w, 's, D: QueryData>(items: Vec<D::Item<'w, 's>>) -> Self::Item<'w, 's, D> {
        items
    }

    fn shrink<'wlong: 'wshort, 'wshort, 's, D: QueryData>(
        item: Self::Item<'wlong, 's, D>,
    ) -> Self::Item<'wshort, 's, D> {
        item.into_iter().map(D::shrink).collect()
    }
}
