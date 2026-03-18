use crate::prelude::*;
use bevy_ecs::{
    prelude::*,
    query::{QueryData, QueryFilter},
};
use std::marker::PhantomData;

#[cfg(feature = "type-aliases")]
pub type J<Ref, Data> = Join<Ref, Data>;

pub struct Join<Ref, Data, Filter = ()>(PhantomData<(Ref, Data, Filter)>)
where
    Ref: Joinable + Component,
    Data: QueryData,
    Filter: QueryFilter;
