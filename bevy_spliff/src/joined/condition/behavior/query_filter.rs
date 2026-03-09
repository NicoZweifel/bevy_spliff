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
                &state.target_state,
                fetch.world.last_change_tick(),
                fetch.world.change_tick(),
            );

            fetch
                .iter_joined(entity)
                .map(|mut targets| {
                    targets
                        .try_fold(
                            ControlFlow::Continue(()),
                            |mut flow, (target, target_cell)| {
                                let location = target_cell.location();
                                let (Some(archetype), Some(table)) = (
                                    fetch.world.archetypes().get(location.archetype_id),
                                    fetch.world.storages().tables.get(location.table_id),
                                ) else {
                                    return Some(flow);
                                };

                                if Filter::matches_component_set(&state.target_state, &|id| {
                                    archetype.contains(id)
                                }) {
                                    Filter::set_archetype(
                                        &mut filter_fetch,
                                        &state.target_state,
                                        archetype,
                                        table,
                                    );

                                    if Filter::filter_fetch(
                                        &state.target_state,
                                        &mut filter_fetch,
                                        target,
                                        location.table_row,
                                    ) {
                                        flow = ControlFlow::Break(());
                                    }
                                }

                                Some(flow)
                            },
                        )
                        .is_some_and(|flow| flow.is_break())
                })
                .unwrap_or_default()
        }
    }
}
