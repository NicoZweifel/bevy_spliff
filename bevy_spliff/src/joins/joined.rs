use crate::prelude::*;
use bevy_ecs::{
    archetype::Archetype,
    change_detection::Tick,
    component::{Component, ComponentId, Components},
    entity::Entity,
    prelude::*,
    query::{EcsAccessType, FilteredAccess, QueryData, ReadOnlyQueryData, WorldQuery},
    storage::{Table, TableRow},
    world::unsafe_world_cell::UnsafeWorldCell,
};
use std::marker::PhantomData;

#[cfg(feature = "type-aliases")]
pub type J<Ref, Data> = Joined<Ref, Data>;

pub struct Joined<Ref, Data>
where
    Ref: Joinable + Component,
    Data: QueryData,
{
    _phantom: PhantomData<(Ref, Data)>,
}

pub struct JoinedFetch<'w, Ref, Data: QueryData> {
    world: UnsafeWorldCell<'w>,
    data_state: Data::State,
    _marker: PhantomData<Ref>,
}

impl<'w, Ref, Data: QueryData> FetchJoiner<'w, Ref> for JoinedFetch<'w, Ref, Data>
where
    Ref: Joinable + Component,
{
    fn world(&self) -> UnsafeWorldCell<'w> {
        self.world
    }
}

impl<Ref, Data: QueryData> Clone for JoinedFetch<'_, Ref, Data>
where
    <Data as WorldQuery>::State: Clone,
{
    fn clone(&self) -> Self {
        JoinedFetch {
            world: self.world,
            data_state: self.data_state.clone(),
            _marker: self._marker,
        }
    }
}

unsafe impl<Ref, Data> WorldQuery for Joined<Ref, Data>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData,
    <Data as WorldQuery>::State: Clone,
{
    type Fetch<'w> = JoinedFetch<'w, Ref, Data>;
    type State = JoinedState<Ref, Data>;

    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        JoinedFetch {
            world: fetch.world,
            data_state: fetch.data_state,
            _marker: PhantomData,
        }
    }

    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        state: &'_ Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Fetch<'w> {
        JoinedFetch {
            world,
            data_state: state.data_state.clone(),
            _marker: PhantomData,
        }
    }

    const IS_DENSE: bool = false;

    unsafe fn set_archetype<'w>(
        _: &mut Self::Fetch<'w>,
        _: &'_ Self::State,
        _: &'w Archetype,
        _: &'w Table,
    ) {
    }
    unsafe fn set_table<'w>(_: &mut Self::Fetch<'w>, _: &'_ Self::State, _: &'w Table) {}

    fn update_component_access(state: &Self::State, access: &mut FilteredAccess) {
        state.update_access(access);
    }

    fn init_state(world: &mut World) -> Self::State {
        JoinedState::new(world.register_component::<Ref>(), Data::init_state(world))
    }

    fn get_state(components: &Components) -> Option<Self::State> {
        Some(JoinedState::new(
            components.get_id(std::any::TypeId::of::<Ref>())?,
            Data::get_state(components)?,
        ))
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        set_contains_id(state.ref_id)
    }
}

unsafe impl<Ref, Data> QueryData for Joined<Ref, Data>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData,
    <Data as WorldQuery>::State: Clone,
{
    const IS_READ_ONLY: bool = true;

    const IS_ARCHETYPAL: bool = false;
    type ReadOnly = Self;
    type Item<'w, 's> = <Ref::Mapper as JoinResultMapper>::Item<'w, 's, Data>;

    fn shrink<'wlong: 'wshort, 'wshort, 's>(
        item: Self::Item<'wlong, 's>,
    ) -> Self::Item<'wshort, 's> {
        <Ref::Mapper as JoinResultMapper>::shrink::<'wlong, 'wshort, 's, Data>(item)
    }

    unsafe fn fetch<'w, 's>(
        state: &'s Self::State,
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        _table_row: TableRow,
    ) -> Option<Self::Item<'w, 's>> {
        unsafe {
            let mut data_fetch = Data::init_fetch(
                fetch.world,
                &state.data_state,
                fetch.world.last_change_tick(),
                fetch.world.change_tick(),
            );

            let res = fetch
                .iter_joined(entity)?
                .map(|(target, cell)| {
                    let location = cell.location();
                    let tables = fetch.world.storages().tables.get(location.table_id)?;
                    let archetype = fetch.world.archetypes().get(location.archetype_id)?;

                    Some(
                        Data::matches_component_set(&state.data_state, &|id| {
                            archetype.contains(id)
                        })
                        .then(|| {
                            Data::set_archetype(
                                &mut data_fetch,
                                &state.data_state,
                                archetype,
                                tables,
                            );

                            Data::fetch(
                                &state.data_state,
                                &mut data_fetch,
                                target,
                                location.table_row,
                            )
                        })
                        .flatten(),
                    )
                })
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .flatten()
                .collect();

            Some(Ref::Mapper::map_results(res))
        }
    }
    #[inline(always)]
    fn iter_access(state: &Self::State) -> impl Iterator<Item = EcsAccessType<'_>> {
        state.iter_access()
    }
}

unsafe impl<Ref, Data> ReadOnlyQueryData for Joined<Ref, Data>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData,
    <Data as WorldQuery>::State: Clone,
{
}
