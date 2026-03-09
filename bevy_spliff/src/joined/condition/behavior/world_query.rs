use crate::prelude::*;
use bevy_ecs::{
    archetype::Archetype,
    change_detection::Tick,
    component::{Component, ComponentId, Components},
    prelude::World,
    query::{FilteredAccess, QueryFilter, WorldQuery},
    storage::Table,
    world::unsafe_world_cell::UnsafeWorldCell,
};

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
        JoinConditionFetch::new(world, state.filter_state.clone())
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
