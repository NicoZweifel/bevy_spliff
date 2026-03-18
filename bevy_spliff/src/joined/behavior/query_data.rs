use std::iter;

use crate::prelude::*;
use bevy_ecs::{
    prelude::*,
    query::{
        ArchetypeQueryData, EcsAccessLevel, EcsAccessType, IterQueryData, NestedQuery, QueryData,
        QueryFilter, ReadOnlyQueryData,
    },
    storage::TableRow,
};

unsafe impl<Ref, Data, Filter> IterQueryData for Joined<Ref, Data, Filter>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData + 'static,
    Filter: QueryFilter + 'static,
{
}

// SAFETY: Joined is read-only because Data is restricted to ReadOnlyQueryData
unsafe impl<Ref, Data, Filter> ReadOnlyQueryData for Joined<Ref, Data, Filter>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData + 'static,
    Filter: QueryFilter + 'static,
{
}

impl<Ref, Data, Filter> ArchetypeQueryData for Joined<Ref, Data, Filter>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData + 'static,
    Filter: QueryFilter + 'static,
{
}

unsafe impl<Ref, Data, Filter> QueryData for Joined<Ref, Data, Filter>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData + 'static,
    Filter: QueryFilter + 'static,
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

    #[inline(always)]
    unsafe fn fetch<'w, 's>(
        (_, state): &'s Self::State,
        fetch: &mut Self::Fetch<'w>,
        entity: Entity,
        _table_row: TableRow,
    ) -> Option<Self::Item<'w, 's>> {
        let r = unsafe { fetch.world.get_entity(entity).ok()?.get::<Ref>()? };
        let Self::Fetch {
            world,
            last_run,
            this_run,
            ..
        } = fetch;

        let res = r.targets().filter_map(move |target| unsafe {
            state
                .query_unchecked_manual_with_ticks(*world, *last_run, *this_run)
                .get_inner(target)
                .ok()
        });

        Some(Ref::Mapper::map_results(res.collect()))
    }

    #[inline(always)]
    fn iter_access((id, state): &Self::State) -> impl Iterator<Item = EcsAccessType<'_>> {
        iter::once(EcsAccessType::Component(EcsAccessLevel::Read(*id))).chain(NestedQuery::<
            Data,
            Filter,
        >::iter_access(
            state
        ))
    }
}
