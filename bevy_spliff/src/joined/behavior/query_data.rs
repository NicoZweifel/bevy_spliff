use crate::prelude::*;
use bevy_ecs::{
    prelude::*,
    query::{EcsAccessType, QueryData, ReadOnlyQueryData, WorldQuery},
    storage::TableRow,
};

unsafe impl<Ref, Data> ReadOnlyQueryData for Joined<Ref, Data>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData,
    <Data as WorldQuery>::State: Clone,
{
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
                &state.target_state,
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
                        Data::matches_component_set(&state.target_state, &|id| {
                            archetype.contains(id)
                        })
                        .then(|| {
                            Data::set_archetype(
                                &mut data_fetch,
                                &state.target_state,
                                archetype,
                                tables,
                            );

                            Data::fetch(
                                &state.target_state,
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
