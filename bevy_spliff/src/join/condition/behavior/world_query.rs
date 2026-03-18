use crate::prelude::*;
use bevy_ecs::{
    archetype::Archetype,
    change_detection::Tick,
    component::{Component, ComponentId, Components},
    prelude::World,
    query::{FilteredAccess, NestedQuery, NestedQueryFetch, QueryFilter, QueryState, WorldQuery},
    storage::Table,
    world::unsafe_world_cell::UnsafeWorldCell,
};

unsafe impl<Ref, Filter> WorldQuery for JoinCondition<Ref, Filter>
where
    Ref: Joinable + Component,
    Filter: QueryFilter + 'static,
{
    type Fetch<'w> = NestedQueryFetch<'w>;
    type State = (ComponentId, QueryState<(), Filter>);

    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        NestedQuery::<(), Filter>::shrink_fetch(fetch)
    }

    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        (_, state): &'_ Self::State,
        last_run: Tick,
        this_run: Tick,
    ) -> Self::Fetch<'w> {
        unsafe { NestedQuery::<(), Filter>::init_fetch(world, state, last_run, this_run) }
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

    fn update_component_access((id, state): &Self::State, access: &mut FilteredAccess) {
        access.add_read(*id);
        NestedQuery::<(), Filter>::update_component_access(state, access);
    }

    fn init_nested_access(
        (_, state): &Self::State,
        system_name: Option<&str>,
        component_access_set: &mut bevy_ecs::query::FilteredAccessSet,
        world: UnsafeWorldCell,
    ) {
        state.init_access(system_name, component_access_set, world);
    }

    fn init_state(world: &mut World) -> Self::State {
        (
            world.register_component::<Ref>(),
            NestedQuery::<(), Filter>::init_state(world),
        )
    }

    fn get_state(_: &Components) -> Option<Self::State> {
        None
    }

    fn matches_component_set(
        (id, _): &Self::State,
        set_contains_id: &impl Fn(ComponentId) -> bool,
    ) -> bool {
        set_contains_id(*id)
    }
}
