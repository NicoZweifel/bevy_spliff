use bevy_ecs::{name::Name, prelude::*};
use bevy_spliff::prelude::*;

mod common;
use common::*;

#[test]
fn join_condition_should_continue_searching() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(InventoryItems[
            Name::new(ITEM_NAME),
            (Name::new(LEGENDARY_NAME), Legendary),
        ]),
    ));

    // Act
    let res: Vec<Entity> = world
        .query_filtered::<Entity, (With<Character>, JC<InventoryItems, With<Legendary>>)>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
}

#[test]
fn join_condition_empty_should_yield_nothing() {
    // Arrange
    let mut world = World::new();
    world.spawn((Name::new(EMPTY_NAME), InventoryItems::default()));

    // Act
    let res: Vec<Entity> = world
        .query_filtered::<Entity, JC<InventoryItems, With<Name>>>()
        .iter(&world)
        .collect();

    // Assert
    assert!(res.is_empty());
}

#[test]
fn join_condition_with_no_matches_should_filter_out_root() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(InventoryItems[
            Name::new(ITEM_NAME),
            Name::new(ITEM_NAME),
        ]),
    ));

    // Act
    let res: Vec<_> = world
        .query_filtered::<Entity, JC<InventoryItems, With<Legendary>>>()
        .iter(&world)
        .collect();

    // Assert
    assert!(res.is_empty());
}
