use crate::prelude::*;
use bevy_ecs::{
    prelude::*,
    query::{QueryFilter, WorldQuery},
    storage::TableRow,
};
use std::ops::ControlFlow;

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
