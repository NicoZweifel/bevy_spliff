use crate::prelude::*;
use bevy_ecs::{prelude::*, query::QueryData};
use std::marker::PhantomData;

#[cfg(feature = "type-aliases")]
pub type J<Ref, Data> = Joined<Ref, Data>;

pub struct Joined<Ref, Data>(PhantomData<(Ref, Data)>)
where
    Ref: Joinable + Component,
    Data: QueryData;
