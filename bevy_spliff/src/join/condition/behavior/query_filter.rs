use crate::prelude::*;
use bevy_ecs::{prelude::*, query::QueryFilter, storage::TableRow};

unsafe impl<Ref, Filter> QueryFilter for JoinCondition<Ref, Filter>
where
    Ref: Joinable + Component,
    Filter: QueryFilter + 'static,
{
    const IS_ARCHETYPAL: bool = false;

    unsafe fn filter_fetch(
        (_, state): &Self::State,
        fetch: &mut Self::Fetch<'_>,
        entity: Entity,
        _: TableRow,
    ) -> bool {
        let Self::Fetch {
            world,
            last_run,
            this_run,
            ..
        } = fetch;

        let Ok(entity_cell) = world.get_entity(entity) else {
            return false;
        };
        let Some(r) = (unsafe { entity_cell.get::<Ref>() }) else {
            return false;
        };

        r.targets().any(move |target| unsafe {
            state
                .query_unchecked_manual_with_ticks(*world, *last_run, *this_run)
                .get_inner(target)
                .is_ok()
        })
    }
}
