use crate::core::prelude::*;
use bevy_ecs::prelude::*;
use std::iter;

impl Joinable for ChildOf {
    type Out<'a> = iter::Once<Entity>;
    type Mapper = SingleJoin;
    fn targets(&self) -> Self::Out<'_> {
        iter::once(self.0)
    }
}

impl Joinable for Children {
    type Out<'a> = iter::Copied<std::slice::Iter<'a, Entity>>;
    type Mapper = MultipleJoin;
    fn targets(&self) -> Self::Out<'_> {
        self.iter()
    }
}
