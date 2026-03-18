use crate::prelude::*;
use bevy_ecs::query::{QueryData, QueryFilter};
use std::marker::PhantomData;

#[cfg(feature = "type-aliases")]
pub type JF<Ref, Data, Filter = ()> = JoinFirst<Ref, Data, Filter>;

pub struct JoinFirst<Ref, Data, Filter = ()>(PhantomData<(Ref, Data, Filter)>)
where
    Ref: Joinable,
    Data: QueryData,
    Filter: QueryFilter;
