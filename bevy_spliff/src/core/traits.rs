use bevy_ecs::{prelude::*, query::QueryData};

pub trait JoinResultMapper: 'static + Send + Sync {
    type Item<'w, 's, D: QueryData>;

    fn map_results<'w, 's, D: QueryData>(items: Vec<D::Item<'w, 's>>) -> Self::Item<'w, 's, D>;

    fn shrink<'wlong: 'wshort, 'wshort, 's, D: QueryData>(
        item: Self::Item<'wlong, 's, D>,
    ) -> Self::Item<'wshort, 's, D>;
}

pub trait Joinable: Component {
    type Out<'a>: Iterator<Item = Entity>
    where
        Self: 'a;
    type Mapper: JoinResultMapper;

    fn targets(&self) -> Self::Out<'_>;
}
