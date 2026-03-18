use bevy_ecs::{name::Name, prelude::*};
use bevy_spliff::prelude::*;

mod common;
use common::*;

#[test]
fn join_should_yield_all() {
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
        .query_filtered::<(&Name, J<InventoryItems, &Name>), With<Character>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    let (player_name, item_names) = &res[0];
    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(item_names.len(), 2);
    assert_eq!(item_names[0].as_str(), ITEM_NAME);
    assert_eq!(item_names[1].as_str(), ITEM_NAME);
}

#[test]
fn join_should_yield_all_children() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Name::new(PARENT_NAME),
        related!(Children[
            Name::new(CHILD_1_NAME),
            Name::new(CHILD_2_NAME),
        ]),
    ));

    // Act
    let res: Vec<Vec<&Name>> = world
        .query_filtered::<J<Children, &Name>, With<Children>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].len(), 2);
    assert_eq!(res[0][0].as_str(), CHILD_1_NAME);
    assert_eq!(res[0][1].as_str(), CHILD_2_NAME);
}

#[test]
fn join_should_yield_deeply_nested_single() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(
            StorageItems[(
                Name::new(VAULT_NAME),
                related!(
                    InventoryItems[(
                        Name::new(CONTAINER_NAME),
                        related!(Weapons[
                            (Name::new(LEGENDARY_NAME), Legendary),
                        ])
                    )]
                )
            )]
        ),
    ));

    // Act
    let mut query = world.query_filtered::<(
        &Name,
        J<StorageItems, (&Name, J<InventoryItems, (&Name, J<Weapons, &Name>)>)>,
    ), (
        With<Character>,
        JC<StorageItems, JC<InventoryItems, JC<Weapons, With<Legendary>>>>,
    )>();
    let res = query.iter(&world).collect::<Vec<_>>();

    // Assert
    assert_eq!(res.len(), 1);
    let (player_name, storages) = &res[0];
    let (vault_name, inventories) = &storages[0];
    let (backpack_name, weapons) = &inventories[0];
    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(vault_name.as_str(), VAULT_NAME);
    assert_eq!(backpack_name.as_str(), CONTAINER_NAME);
    assert_eq!(weapons.len(), 1);
    assert_eq!(weapons[0].as_str(), LEGENDARY_NAME);
}

#[test]
fn join_should_yield_deeply_nested_filtered() {
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
            J<InventoryItems, (&Name, J<Weapons, (&Name, &Legendary)>)>,
        ), (
            With<Character>,
            JC<InventoryItems, JC<Weapons, With<Legendary>>>,
        )>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    let (player_name, inventories) = &res[0];
    let (bag_name, weapons) = &inventories[0];
    let weapon_names: Vec<&str> = weapons.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(bag_name.as_str(), CONTAINER_NAME);
    assert_eq!(weapon_names.len(), 1);
    assert_eq!(weapon_names[0], LEGENDARY_NAME);
}

#[test]
fn join_should_yield_all_deeply_nested_filtered() {
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
        .query_filtered::<(&Name, J<InventoryItems, (&Name, J<Weapons, &Name>)>), (
            With<Character>,
            JC<InventoryItems, JC<Weapons, With<Legendary>>>,
        )>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    let (player_name, inventories) = &res[0];
    let (bag_name, weapons) = &inventories[0];
    let weapon_names: Vec<&str> = weapons.iter().map(|n| n.as_str()).collect();

    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(bag_name.as_str(), CONTAINER_NAME);
    assert_eq!(weapon_names.len(), 2);
    assert!(weapon_names.contains(&LEGENDARY_NAME));
    assert!(weapon_names.contains(&WEAPON_NAME));
}

#[test]
fn join_should_skip_empty_and_yield_valid() {
    // Arrange
    let mut world = World::new();
    let valid = world.spawn(Name::new(VALID_NAME)).id();
    let invalid = world.spawn_empty().id();
    world.spawn(InventoryItems::new(vec![valid, invalid]));

    // Act
    let res: Vec<Vec<&Name>> = world
        .query::<J<InventoryItems, &Name>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res[0].len(), 1);
    assert_eq!(res[0][0].as_str(), VALID_NAME);
}

#[test]
fn join_should_skip_despawned() {
    // Arrange
    let mut world = World::new();
    let e = world.spawn(Name::new(GHOST_NAME)).id();
    world.spawn(InventoryItems::new(vec![e]));
    world.despawn(e);

    // Act
    let res: Vec<Vec<&Name>> = world
        .query::<J<InventoryItems, &Name>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert!(res[0].is_empty());
}

#[test]
fn join_should_yield_empty() {
    // Arrange
    let mut world = World::new();
    world.spawn(InventoryItems::default());

    // Act
    let res: Vec<_> = world
        .query::<J<InventoryItems, &Name>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert!(res[0].is_empty());
}

#[test]
fn join_optional_should_yield_none() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(InventoryItems[
            (Name::new(LEGENDARY_NAME), Legendary),
            Name::new(ITEM_NAME),
        ]),
    ));

    // Act
    let res: Vec<_> = world
        .query::<J<InventoryItems, (&Name, Option<&Legendary>)>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    let targets = &res[0];
    assert_eq!(targets.len(), 2);
    assert!(targets[0].1.is_some());
    assert!(targets[1].1.is_none());
}
