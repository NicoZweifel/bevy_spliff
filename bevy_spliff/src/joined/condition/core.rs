use crate::prelude::*;
use bevy_ecs::{
    component::ComponentId,
    prelude::*,
    query::{QueryFilter, WorldQuery},
    world::unsafe_world_cell::UnsafeWorldCell,
};
use std::marker::PhantomData;

#[cfg(feature = "type-aliases")]
pub type JC<Ref, Filter> = JoinCondition<Ref, Filter>;

pub struct JoinCondition<Ref, Filter>
where
    Ref: Joinable,
    Filter: QueryFilter,
{
    _phantom: PhantomData<(Ref, Filter)>,
}

pub struct JoinConditionState<Ref, Filter: QueryFilter> {
    pub ref_id: ComponentId,
    pub filter_state: Filter::State,
    _marker: PhantomData<Ref>,
}

impl<Ref, Filter: QueryFilter> JoinConditionState<Ref, Filter> {
    pub fn new(ref_id: ComponentId, filter_state: Filter::State) -> Self {
        Self {
            ref_id,
            filter_state,
            _marker: PhantomData,
        }
    }
}

impl<Ref, Filter: QueryFilter> Clone for JoinConditionState<Ref, Filter>
where
    Filter::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            ref_id: self.ref_id,
            filter_state: self.filter_state.clone(),
            _marker: PhantomData,
        }
    }
}

pub struct JoinConditionFetch<'w, Ref, Filter: QueryFilter> {
    pub(crate) world: UnsafeWorldCell<'w>,
    filter_state: Filter::State,
    _marker: PhantomData<(Ref, Filter)>,
}

impl<'w, Ref, Filter: QueryFilter> JoinConditionFetch<'w, Ref, Filter> {
    pub fn new(world: UnsafeWorldCell<'w>, filter_state: Filter::State) -> Self {
        Self {
            world,
            filter_state,
            _marker: PhantomData,
        }
    }
}

impl<'w, Ref, Filter: QueryFilter> FetchJoiner<'w, Ref> for JoinConditionFetch<'w, Ref, Filter>
where
    Ref: Joinable + Component,
{
    fn world(&self) -> UnsafeWorldCell<'w> {
        self.world
    }
}

impl<Ref, Filter: QueryFilter> Clone for JoinConditionFetch<'_, Ref, Filter>
where
    <Filter as WorldQuery>::State: Clone,
{
    fn clone(&self) -> Self {
        JoinConditionFetch {
            world: self.world,
            filter_state: self.filter_state.clone(),
            _marker: PhantomData,
        }
    }
}
