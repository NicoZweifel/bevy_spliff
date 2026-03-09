use crate::prelude::*;
use bevy_ecs::{
    archetype::Archetype,
    change_detection::Tick,
    component::{Component, ComponentId, Components},
    prelude::*,
    query::{FilteredAccess, ReadOnlyQueryData, WorldQuery},
    storage::Table,
    world::unsafe_world_cell::UnsafeWorldCell,
};

unsafe impl<Ref, Data> WorldQuery for Joined<Ref, Data>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData,
    <Data as WorldQuery>::State: Clone,
{
    type Fetch<'w> = JoinedFetch<'w, Ref, Data>;
    type State = JoinedState<Ref, Data>;

    fn shrink_fetch<'wlong: 'wshort, 'wshort>(fetch: Self::Fetch<'wlong>) -> Self::Fetch<'wshort> {
        JoinedFetch::new(fetch.world, fetch.target_state)
    }

    unsafe fn init_fetch<'w>(
        world: UnsafeWorldCell<'w>,
        state: &'_ Self::State,
        _last_run: Tick,
        _this_run: Tick,
    ) -> Self::Fetch<'w> {
        JoinedFetch::new(world, state.target_state.clone())
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
