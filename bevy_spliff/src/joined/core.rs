use crate::prelude::*;
use bevy_ecs::{
    component::ComponentId,
    prelude::*,
    query::{QueryData, WorldQuery},
    world::unsafe_world_cell::UnsafeWorldCell,
};
use std::marker::PhantomData;

#[cfg(feature = "type-aliases")]
pub type J<Ref, Data> = Joined<Ref, Data>;

pub struct Joined<Ref, Data>(PhantomData<(Ref, Data)>)
where
    Ref: Joinable + Component,
    Data: QueryData;

pub struct JoinedFetch<'w, Ref, Data: QueryData> {
    pub world: UnsafeWorldCell<'w>,
    pub data_state: Data::State,
    _marker: PhantomData<Ref>,
}

impl<'w, Ref, Data: QueryData> JoinedFetch<'w, Ref, Data> {
    pub fn new(world: UnsafeWorldCell<'w>, data_state: Data::State) -> Self {
        Self {
            world,
            data_state,
            _marker: PhantomData,
        }
    }
}

impl<Ref, Data: QueryData> Clone for JoinedFetch<'_, Ref, Data>
where
    <Data as WorldQuery>::State: Clone,
{
    fn clone(&self) -> Self {
        JoinedFetch {
            world: self.world,
            data_state: self.data_state.clone(),
            _marker: self._marker,
        }
    }
}

impl<'w, Ref, Data: QueryData> FetchJoiner<'w, Ref> for JoinedFetch<'w, Ref, Data>
where
    Ref: Joinable + Component,
{
    fn world(&self) -> UnsafeWorldCell<'w> {
        self.world
    }
}

pub struct JoinedState<Ref, Data: QueryData> {
    pub ref_id: ComponentId,
    pub data_state: Data::State,
    _marker: PhantomData<Ref>,
}

impl<Ref, Data: QueryData> JoinedState<Ref, Data> {
    pub fn new(ref_id: ComponentId, data_state: Data::State) -> Self {
        Self {
            ref_id,
            data_state,
            _marker: PhantomData,
        }
    }
}

impl<Ref, Data: QueryData> Clone for JoinedState<Ref, Data>
where
    Data::State: Clone,
{
    fn clone(&self) -> Self {
        Self {
            ref_id: self.ref_id,
            data_state: self.data_state.clone(),
            _marker: PhantomData,
        }
    }
}

impl<Ref, Data: QueryData> JoinState for JoinedState<Ref, Data> {
    type Data = Data;
    fn ref_id(&self) -> ComponentId {
        self.ref_id
    }
    fn data_state(&self) -> &Data::State {
        &self.data_state
    }
}
