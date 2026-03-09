use bevy_ecs::{
    component::ComponentId,
    prelude::*,
    query::{EcsAccessLevel, EcsAccessType, QueryData, WorldQuery},
    world::unsafe_world_cell::{UnsafeEntityCell, UnsafeWorldCell},
};
use std::iter;



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

pub(crate) trait FetchJoiner<'w, Ref: Joinable + Component> {
    fn world(&self) -> UnsafeWorldCell<'w>;

    #[inline(always)]
    unsafe fn iter_joined(
        &self,
        entity: Entity,
    ) -> Option<impl Iterator<Item = (Entity, UnsafeEntityCell<'w>)>> {
        unsafe {
            let r = self.world().get_entity(entity).ok()?.get::<Ref>()?;
            Some(r.targets().filter_map(|target| {
                self.world()
                    .get_entity(target)
                    .ok()
                    .map(|cell| (target, cell))
            }))
        }
    }
}

pub(crate) trait JoinState {
    type Data: QueryData;
    fn ref_id(&self) -> ComponentId;
    fn target_state(&self) -> &<Self::Data as WorldQuery>::State;

    #[inline(always)]
    fn iter_access(&self) -> impl Iterator<Item = EcsAccessType<'_>> {
        iter::once(EcsAccessType::Component(EcsAccessLevel::Read(
            self.ref_id(),
        )))
        .chain(Self::Data::iter_access(self.target_state()))
    }
}
