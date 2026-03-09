use bevy_ecs::{
    component::{Component, ComponentId},
    entity::Entity,
    prelude::*,
    query::{EcsAccessLevel, EcsAccessType, FilteredAccess, QueryData, WorldQuery},
    world::unsafe_world_cell::{UnsafeEntityCell, UnsafeWorldCell},
};
use std::{iter, marker::PhantomData};

pub trait JoinResultMapper: 'static + Send + Sync {
    type Item<'w, 's, D: QueryData>;

    fn map_results<'w, 's, D: QueryData>(items: Vec<D::Item<'w, 's>>) -> Self::Item<'w, 's, D>;

    fn shrink<'wlong: 'wshort, 'wshort, 's, D: QueryData>(
        item: Self::Item<'wlong, 's, D>,
    ) -> Self::Item<'wshort, 's, D>;
}

pub struct SingleJoin;

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

pub struct MultipleJoin;

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

pub trait Joinable: Component {
    type Out<'a>: Iterator<Item = Entity>
    where
        Self: 'a;
    type Mapper: JoinResultMapper;

    fn targets(&self) -> Self::Out<'_>;
}

impl Joinable for ChildOf {
    type Out<'a> = iter::Once<Entity>;
    type Mapper = SingleJoin;
    fn targets(&self) -> Self::Out<'_> {
        iter::once(self.0)
    }
}

impl Joinable for Children {
    type Out<'a> = iter::Copied<std::slice::Iter<'a, Entity>>;
    type Mapper = MultipleJoin;
    fn targets(&self) -> Self::Out<'_> {
        self.iter()
    }
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
    fn data_state(&self) -> &<Self::Data as WorldQuery>::State;

    #[inline(always)]
    fn update_access(&self, access: &mut FilteredAccess) {
        access.add_component_read(self.ref_id());
        let mut data_access = FilteredAccess::default();
        Self::Data::update_component_access(self.data_state(), &mut data_access);
        access.access_mut().extend(data_access.access());
    }

    #[inline(always)]
    fn iter_access(&self) -> impl Iterator<Item = EcsAccessType<'_>> {
        iter::once(EcsAccessType::Component(EcsAccessLevel::Read(
            self.ref_id(),
        )))
        .chain(Self::Data::iter_access(self.data_state()))
    }
}

pub struct JoinedState<Ref, Data: QueryData> {
    pub ref_id: ComponentId,
    pub data_state: Data::State,
    _marker: PhantomData<Ref>,
}

impl<Ref, Data: QueryData> JoinedState<Ref, Data> {
    pub fn new(ref_id: ComponentId, data_state: Data::State) -> Self {
        Self {
            ref_id,
            data_state,
            _marker: PhantomData,
        }
    }
}

impl<Ref, Data: QueryData> JoinState for JoinedState<Ref, Data> {
    type Data = Data;
    fn ref_id(&self) -> ComponentId {
        self.ref_id
    }
    fn data_state(&self) -> &Data::State {
        &self.data_state
    }
}

impl<Ref, Data: QueryData> Clone for JoinedState<Ref, Data>
where
    Data::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            ref_id: self.ref_id,
            data_state: self.data_state.clone(),
            _marker: PhantomData,
        }
    }
}
