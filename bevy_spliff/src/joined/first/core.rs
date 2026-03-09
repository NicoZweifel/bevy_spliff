use crate::prelude::*;
use bevy_ecs::query::QueryData;
use std::marker::PhantomData;

#[cfg(feature = "type-aliases")]
pub type JF<Ref, Data> = JoinedFirst<Ref, Data>;

pub struct JoinedFirst<Ref, Data>(PhantomData<(Ref, Data)>)
where
    Ref: Joinable,
    Data: QueryData;
