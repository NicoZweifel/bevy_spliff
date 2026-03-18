use bevy_ecs::{name::Name, prelude::*};
use bevy_spliff::prelude::*;

mod common;
use common::*;

#[test]
fn joined_first_should_filter() {
    // Arrange
    let mut world = World::new();
    world.spawn((Character, related!(InventoryItems[Name::new(ITEM_NAME)])));
    world.spawn((Character, Name::new(ENEMY_NAME)));

    // Act
    let res: Vec<&Name> = world
        .query_filtered::<(&Name, JF<InventoryItems, &Name>), ()>()
        .iter(&world)
        .map(|x| x.0)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].as_str(), PLAYER_NAME);
}

#[test]
fn joined_first_component_should_filter() {
    // Arrange
    let mut world = World::new();
    world.spawn((Character, related!(InventoryItems[Name::new(ITEM_NAME)])));
    world.spawn((
        Character,
        Name::new(ENEMY_NAME),
        related!(InventoryItems[(Name::new(LEGENDARY_NAME), Legendary)]),
    ));

    // Act
    let res: Vec<&Name> = world
        .query::<(&Name, JF<InventoryItems, &Legendary>)>()
        .iter(&world)
        .map(|x| x.0)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].as_str(), ENEMY_NAME);
}

#[test]
fn joined_first_empty_should_filter_out_root() {
    // Arrange
    let mut world = World::new();
    world.spawn((Name::new(UNARMED_NAME), InventoryItems::default()));

    // Act
    let res: Vec<_> = world
        .query::<JF<InventoryItems, &Name>>()
        .iter(&world)
        .collect();

    // Assert
    assert!(res.is_empty());
}

#[test]
fn joined_first_should_skip_invalid_targets_and_yield_first_valid() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(InventoryItems[
            Name::new(ITEM_NAME),
            (Name::new(LEGENDARY_NAME), Legendary),
            (Name::new("Another"), Legendary),
        ]),
    ));

    // Act
    let res: Vec<_> = world
        .query::<(&Name, JF<InventoryItems, (&Name, &Legendary)>)>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    let (player, (item_name, _)) = res[0];
    assert_eq!(player.as_str(), PLAYER_NAME);
    assert_eq!(item_name.as_str(), LEGENDARY_NAME);
}

#[test]
fn joined_first_deeply_nested_filtered_should_yield_single_match() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(
            InventoryItems[(
                Name::new(CONTAINER_NAME),
                related!(Weapons[
                    (Name::new(LEGENDARY_NAME), Legendary),
                    Name::new(WEAPON_NAME),
                ])
            )]
        ),
    ));

    // Act
    let res: Vec<_> = world
        .query_filtered::<(
            &Name,
            JF<InventoryItems, (&Name, JF<Weapons, (&Name, &Legendary)>)>,
        ), (
            With<Character>,
            JC<InventoryItems, JC<Weapons, With<Legendary>>>,
        )>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    let (player_name, (bag_name, (weapon_name, _))) = res[0];
    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(bag_name.as_str(), CONTAINER_NAME);
    assert_eq!(weapon_name.as_str(), LEGENDARY_NAME);
}

#[test]
fn joined_first_deeply_nested_filtered_should_yield_all() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(
            InventoryItems[(
                Name::new(CONTAINER_NAME),
                related!(Weapons[
                    (Name::new(LEGENDARY_NAME), Legendary),
                    Name::new(WEAPON_NAME),
                ])
            )]
        ),
    ));

    // Act
    let res: Vec<_> = world
        .query_filtered::<(&Name, JF<InventoryItems, (&Name, J<Weapons, &Name>)>), (
            With<Character>,
            JC<InventoryItems, JC<Weapons, With<Legendary>>>,
        )>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    let (player_name, (bag_name, weapons)) = &res[0];
    let weapon_names: Vec<&str> = weapons.iter().map(|n| n.as_str()).collect();

    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(bag_name.as_str(), CONTAINER_NAME);
    assert_eq!(weapon_names.len(), 2);
    assert!(weapon_names.contains(&LEGENDARY_NAME));
    assert!(weapon_names.contains(&WEAPON_NAME));
}
