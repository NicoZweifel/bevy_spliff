use crate::prelude::*;
use bevy_ecs::query::{QueryData, QueryFilter};
use std::marker::PhantomData;

#[cfg(feature = "type-aliases")]
pub type JF<Ref, Data, Filter = ()> = JoinedFirst<Ref, Data, Filter>;

pub struct JoinedFirst<Ref, Data, Filter = ()>(PhantomData<(Ref, Data, Filter)>)
where
    Ref: Joinable,
    Data: QueryData,
    Filter: QueryFilter;
