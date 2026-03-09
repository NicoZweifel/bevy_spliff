use crate::prelude::*;
use bevy_ecs::{
    prelude::*,
    query::{EcsAccessType, QueryData, ReadOnlyQueryData, WorldQuery},
    storage::TableRow,
};
use std::ops::ControlFlow;

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
