use crate::prelude::*;
use bevy_ecs::{
    archetype::Archetype,
    change_detection::Tick,
    component::{ComponentId, Components},
    prelude::*,
    query::{EcsAccessType, FilteredAccess, QueryData, ReadOnlyQueryData, WorldQuery},
    storage::{Table, TableRow},
    world::unsafe_world_cell::UnsafeWorldCell,
};
use std::{marker::PhantomData, ops::ControlFlow};

#[cfg(feature = "type-aliases")]
pub type JF<Ref, Data> = JoinedFirst<Ref, Data>;

pub struct JoinedFirst<Ref, Data>
where
    Ref: Joinable,
    Data: QueryData,
{
    _phantom: PhantomData<(Ref, Data)>,
}

unsafe impl<Ref, Data> WorldQuery for JoinedFirst<Ref, Data>
where
    Ref: Joinable + Component,
    Data: QueryData,
    <Data as WorldQuery>::State: Clone,
{
    type Fetch<'w> = JoinedFirstFetch<'w, Ref, Data>;
    type State = JoinedState<Ref, Data>;

    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        JoinedFirstFetch {
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
        JoinedFirstFetch {
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

unsafe impl<Ref, Data> QueryData for JoinedFirst<Ref, Data>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData,
    <Data as WorldQuery>::State: Clone,
{
    const IS_READ_ONLY: bool = true;
    const IS_ARCHETYPAL: bool = false;

    type ReadOnly = Self;
    type Item<'w, 's> = Data::Item<'w, 's>;

    fn shrink<'wlong: 'wshort, 'wshort, 's>(
        item: Self::Item<'wlong, 's>,
    ) -> Self::Item<'wshort, 's> {
        Data::shrink::<'wlong, 'wshort, 's>(item)
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

            fetch
                .iter_joined(entity)?
                .try_fold(ControlFlow::Continue(()), |_, (target, target_cell)| {
                    let location = target_cell.location();
                    let archetype = fetch.world.archetypes().get(location.archetype_id)?;
                    let table = fetch.world.storages().tables.get(location.table_id)?;

                    if Data::matches_component_set(&state.data_state, &|id| archetype.contains(id))
                    {
                        Data::set_archetype(&mut data_fetch, &state.data_state, archetype, table);
                        Data::fetch(
                            &state.data_state,
                            &mut data_fetch,
                            target,
                            location.table_row,
                        )
                        .map(ControlFlow::Break)
                    } else {
                        Some(ControlFlow::Continue(()))
                    }
                })?
                .break_value()
        }
    }

    #[inline(always)]
    fn iter_access(state: &Self::State) -> impl Iterator<Item = EcsAccessType<'_>> {
        state.iter_access()
    }
}

unsafe impl<Ref, Data> ReadOnlyQueryData for JoinedFirst<Ref, Data>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData,
    <Data as WorldQuery>::State: Clone,
{
}

pub struct JoinedFirstFetch<'w, Ref, Data: QueryData> {
    world: UnsafeWorldCell<'w>,
    data_state: Data::State,
    _marker: PhantomData<Ref>,
}

impl<'w, Ref, Data: QueryData> FetchJoiner<'w, Ref> for JoinedFirstFetch<'w, Ref, Data>
where
    Ref: Joinable + Component,
{
    fn world(&self) -> UnsafeWorldCell<'w> {
        self.world
    }
}

impl<Ref, Data: QueryData> Clone for JoinedFirstFetch<'_, Ref, Data>
where
    <Data as WorldQuery>::State: Clone,
{
    fn clone(&self) -> Self {
        JoinedFirstFetch {
            world: self.world,
            data_state: self.data_state.clone(),
            _marker: self._marker,
        }
    }
}
