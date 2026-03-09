use crate::prelude::*;
use bevy_ecs::{
    component::ComponentId,
    prelude::*,
    query::{FilteredAccess, QueryData, WorldQuery},
    world::unsafe_world_cell::UnsafeWorldCell,
};
use std::marker::PhantomData;

pub struct SingleJoin;

pub struct MultipleJoin;

pub struct JoinedFetch<'w, Ref, Target: WorldQuery> {
    pub world: UnsafeWorldCell<'w>,
    pub target_state: Target::State,
    _marker: PhantomData<Ref>,
}

impl<'w, Ref, Target: WorldQuery> JoinedFetch<'w, Ref, Target> {
    pub fn new(world: UnsafeWorldCell<'w>, data_state: Target::State) -> Self {
        Self {
            world,
            target_state: data_state,
            _marker: PhantomData,
        }
    }
}

impl<Ref, Target: WorldQuery> Clone for JoinedFetch<'_, Ref, Target>
where
    <Target as WorldQuery>::State: Clone,
{
    fn clone(&self) -> Self {
        JoinedFetch {
            world: self.world,
            target_state: self.target_state.clone(),
            _marker: self._marker,
        }
    }
}

impl<'w, Ref, Target: WorldQuery> FetchJoiner<'w, Ref> for JoinedFetch<'w, Ref, Target>
where
    Ref: Joinable + Component,
{
    fn world(&self) -> UnsafeWorldCell<'w> {
        self.world
    }
}

pub struct JoinedState<Ref, Target: WorldQuery> {
    pub ref_id: ComponentId,
    pub target_state: Target::State,
    _marker: PhantomData<Ref>,
}

impl<Ref, Target: WorldQuery> JoinedState<Ref, Target> {
    pub fn new(ref_id: ComponentId, target_state: Target::State) -> Self {
        Self {
            ref_id,
            target_state,
            _marker: PhantomData,
        }
    }

    #[inline(always)]
    pub fn update_access(&self, access: &mut FilteredAccess) {
        access.add_component_read(self.ref_id);
        let mut target_access = FilteredAccess::default();
        Target::update_component_access(&self.target_state, &mut target_access);
        access.access_mut().extend(target_access.access());
    }
}

impl<Ref, Target: WorldQuery> Clone for JoinedState<Ref, Target>
where
    Target::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            ref_id: self.ref_id,
            target_state: self.target_state.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Ref, Target: QueryData> JoinState for JoinedState<Ref, Target> {
    type Data = Target;
    fn ref_id(&self) -> ComponentId {
        self.ref_id
    }
    fn target_state(&self) -> &Target::State {
        &self.target_state
    }
}
