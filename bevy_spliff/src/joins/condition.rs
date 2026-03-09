use crate::prelude::*;
use bevy_ecs::{
    archetype::Archetype,
    change_detection::Tick,
    component::{Component, ComponentId, Components},
    entity::Entity,
    prelude::World,
    query::{FilteredAccess, QueryFilter, WorldQuery},
    storage::{Table, TableRow},
    world::unsafe_world_cell::UnsafeWorldCell,
};
use std::marker::PhantomData;
use std::ops::ControlFlow;

#[cfg(feature = "type-aliases")]
pub type JC<Ref, Filter> = JoinCondition<Ref, Filter>;

pub struct JoinCondition<Ref, Filter>
where
    Ref: Joinable,
    Filter: QueryFilter,
{
    _phantom: PhantomData<(Ref, Filter)>,
}

unsafe impl<Ref, Filter> WorldQuery for JoinCondition<Ref, Filter>
where
    Ref: Joinable + Component,
    Filter: QueryFilter,
    <Filter as WorldQuery>::State: Clone,
{
    type Fetch<'w> = JoinConditionFetch<'w, Ref, Filter>;
    type State = JoinConditionState<Ref, Filter>;

    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        fetch
    }

    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        state: &'_ Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Fetch<'w> {
        JoinConditionFetch {
            world,
            filter_state: state.filter_state.clone(),
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
        access.add_component_read(state.ref_id);
    }

    fn init_state(world: &mut World) -> Self::State {
        JoinConditionState::new(world.register_component::<Ref>(), Filter::init_state(world))
    }

    fn get_state(components: &Components) -> Option<Self::State> {
        Some(JoinConditionState::new(
            components.get_id(std::any::TypeId::of::<Ref>())?,
            Filter::get_state(components)?,
        ))
    }

    fn matches_component_set(
        state: &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        set_contains_id(state.ref_id)
    }
}

unsafe impl<Ref, Filter> QueryFilter for JoinCondition<Ref, Filter>
where
    Ref: Joinable + Component,
    Filter: QueryFilter,
    <Filter as WorldQuery>::State: Clone,
{
    const IS_ARCHETYPAL: bool = false;

    unsafe fn filter_fetch(
        state: &Self::State,
        fetch: &mut Self::Fetch<'_>,
        entity: Entity,
        _table_row: TableRow,
    ) -> bool {
        unsafe {
            let mut filter_fetch = Filter::init_fetch(
                fetch.world,
                &state.filter_state,
                fetch.world.last_change_tick(),
                fetch.world.change_tick(),
            );

            fetch
                .iter_joined(entity)
                .map(|mut targets| {
                    targets
                        .try_fold(ControlFlow::Continue(()), |_, (target, target_cell)| {
                            let location = target_cell.location();
                            let archetype = fetch.world.archetypes().get(location.archetype_id)?;
                            let table = fetch.world.storages().tables.get(location.table_id)?;

                            if Filter::matches_component_set(&state.filter_state, &|id| {
                                archetype.contains(id)
                            }) {
                                Filter::set_archetype(
                                    &mut filter_fetch,
                                    &state.filter_state,
                                    archetype,
                                    table,
                                );

                                Filter::filter_fetch(
                                    &state.filter_state,
                                    &mut filter_fetch,
                                    target,
                                    location.table_row,
                                )
                                .then_some(ControlFlow::Break(()))
                            } else {
                                Some(ControlFlow::Continue(()))
                            }
                        })
                        .is_some_and(|flow| flow.is_break())
                })
                .unwrap_or_default()
        }
    }
}

pub struct JoinConditionState<Ref, Filter: QueryFilter> {
    ref_id: ComponentId,
    filter_state: Filter::State,
    _marker: PhantomData<Ref>,
}

impl<Ref, Filter: QueryFilter> JoinConditionState<Ref, Filter> {
    pub fn new(ref_id: ComponentId, filter_state: Filter::State) -> Self {
        Self {
            ref_id,
            filter_state,
            _marker: PhantomData,
        }
    }
}

impl<Ref, Filter: QueryFilter> Clone for JoinConditionState<Ref, Filter>
where
    Filter::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            ref_id: self.ref_id,
            filter_state: self.filter_state.clone(),
            _marker: PhantomData,
        }
    }
}

pub struct JoinConditionFetch<'w, Ref, Filter: QueryFilter> {
    world: UnsafeWorldCell<'w>,
    filter_state: Filter::State,
    _marker: PhantomData<(Ref, Filter)>,
}

impl<'w, Ref, Filter: QueryFilter> FetchJoiner<'w, Ref> for JoinConditionFetch<'w, Ref, Filter>
where
    Ref: Joinable + Component,
{
    fn world(&self) -> UnsafeWorldCell<'w> {
        self.world
    }
}

impl<Ref, Filter: QueryFilter> Clone for JoinConditionFetch<'_, Ref, Filter>
where
    <Filter as WorldQuery>::State: Clone,
{
    fn clone(&self) -> Self {
        JoinConditionFetch {
            world: self.world,
            filter_state: self.filter_state.clone(),
            _marker: PhantomData,
        }
    }
}
