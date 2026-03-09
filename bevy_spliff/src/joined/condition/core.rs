use crate::prelude::*;
use bevy_ecs::query::QueryFilter;
use std::marker::PhantomData;

#[cfg(feature = "type-aliases")]
pub type JC<Ref, Filter> = JoinCondition<Ref, Filter>;

pub struct JoinCondition<Ref, Filter>(PhantomData<(Ref, Filter)>)
where
    Ref: Joinable,
    Filter: QueryFilter;
