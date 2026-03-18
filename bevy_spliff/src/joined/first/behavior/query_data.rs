use crate::prelude::*;
use bevy_ecs::{
    prelude::*,
    query::{
        ArchetypeQueryData, EcsAccessLevel, EcsAccessType, IterQueryData, NestedQuery, QueryData,
        QueryFilter, ReadOnlyQueryData,
    },
    storage::TableRow,
};
use std::iter;

unsafe impl<Ref, Data, Filter> IterQueryData for JoinedFirst<Ref, Data, Filter>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData + 'static,
    Filter: QueryFilter + 'static,
{
}

// SAFETY: JoinedFirst is read-only because Data is restricted to ReadOnlyQueryData
unsafe impl<Ref, Data, Filter> ReadOnlyQueryData for JoinedFirst<Ref, Data, Filter>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData + 'static,
    Filter: QueryFilter + 'static,
{
}

impl<Ref, Data, Filter> ArchetypeQueryData for JoinedFirst<Ref, Data, Filter>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData + 'static,
    Filter: QueryFilter + 'static,
{
}

unsafe impl<Ref, Data, Filter> QueryData for JoinedFirst<Ref, Data, Filter>
where
    Ref: Joinable + Component,
    Data: ReadOnlyQueryData + 'static,
    Filter: QueryFilter + 'static,
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

        r.targets().find_map(move |target| unsafe {
            state
                .query_unchecked_manual_with_ticks(*world, *last_run, *this_run)
                .get_inner(target)
                .ok()
        })
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
