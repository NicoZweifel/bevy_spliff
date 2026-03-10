use bevy_ecs::{name::Name, prelude::*};
use bevy_spliff::prelude::*;

const PLAYER_NAME: &str = "Player";
const ENEMY_NAME: &str = "Enemy";
const ITEM_NAME: &str = "Item";
const CONTAINER_NAME: &str = "Container";
const WEAPON_NAME: &str = "Sword";
const LEGENDARY_NAME: &str = "Excalibur";
const VALID_NAME: &str = "Valid";
const GHOST_NAME: &str = "Ghost";
const PARENT_NAME: &str = "Parent";
const CHILD_1_NAME: &str = "Child 1";
const CHILD_2_NAME: &str = "Child 2";
const EMPTY_NAME: &str = "Empty";
const UNARMED_NAME: &str = "Unarmed";

#[derive(Component)]
#[require(Inventory, Storage, Name::new(PLAYER_NAME))]
struct Character;

#[derive(Component)]
struct Legendary;

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = InInventory)]
struct Inventory(Vec<Entity>);

#[derive(Component, Joinable)]
#[relationship(relationship_target = Inventory)]
struct InInventory(Entity);

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = WeaponOf)]
struct Weapons(Vec<Entity>);

#[derive(Component, Joinable)]
#[relationship(relationship_target = Weapons)]
struct WeaponOf(Entity);

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = InStorage)]
struct Storage(Vec<Entity>);

#[derive(Component, Joinable)]
#[relationship(relationship_target = Storage)]
struct InStorage(Entity);

#[test]
fn joined_one_to_many_should_yield_all() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(Inventory[
            Name::new(ITEM_NAME),
            Name::new(ITEM_NAME),
        ]),
    ));

    // Act
    let results: Vec<_> = world
        .query_filtered::<(&Name, J<Inventory, &Name>), With<Character>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(results.len(), 1);
    let (player_name, item_names) = &results[0];
    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(item_names.len(), 2);
    assert_eq!(item_names[0].as_str(), ITEM_NAME);
    assert_eq!(item_names[1].as_str(), ITEM_NAME);
}

#[test]
fn joined_deeply_nested_single_should_yield_single() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(
            Inventory[(
                Name::new(CONTAINER_NAME),
                related!(Weapons[
                    (Name::new(LEGENDARY_NAME), Legendary),
                ])
            )]
        ),
    ));

    // Act
    let mut query = world.query_filtered::<
        (&Name, J<Inventory, (&Name, J<Weapons, &Name>)>),
        (With<Character>, JC<Inventory, JC<Weapons, With<Legendary>>>)
    >();
    let results = query.iter(&world).collect::<Vec<_>>();

    // Assert
    assert_eq!(results.len(), 1);
    let (player_name, inventories) = &results[0];
    let (bag_name, weapons) = &inventories[0];

    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(bag_name.as_str(), CONTAINER_NAME);
    assert_eq!(weapons.len(), 1);
    assert_eq!(weapons[0].as_str(), LEGENDARY_NAME);
}

#[test]
fn joined_deeply_nested_multiple_should_yield_all() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(
            Storage[(
                Name::new(CONTAINER_NAME),
                related!(Weapons[
                    (Name::new(LEGENDARY_NAME), Legendary),
                ])
            )]
        ),
    ));

    // Act
    let mut query = world.query_filtered::<
        (&Name, J<Storage, (&Name, J<Weapons, &Name>)>),
        (With<Character>, JC<Storage, JC<Weapons, With<Legendary>>>)
    >();
    let results = query.iter(&world).collect::<Vec<_>>();

    // Assert
    assert_eq!(results.len(), 1);
    let (player_name, storages) = &results[0];
    let (bag_name, weapons) = &storages[0];

    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(bag_name.as_str(), CONTAINER_NAME);
    assert_eq!(weapons.len(), 1);
    assert_eq!(weapons[0].as_str(), LEGENDARY_NAME);
}

#[test]
fn joined_deeply_nested_filtered_should_yield_single() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(
            Inventory[(
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
            J<Inventory, (&Name, J<Weapons, (&Name, &Legendary)>)>,
        ), (With<Character>, JC<Inventory, JC<Weapons, With<Legendary>>>)>()
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
fn joined_deeply_nested_filtered_should_yield_all() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(
            Inventory[(
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
        .query_filtered::<(&Name, J<Inventory, (&Name, J<Weapons, &Name>)>), (
            With<Character>,
            JC<Inventory, JC<Weapons, With<Legendary>>>,
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
fn joined_first_deeply_nested_filtered_should_yield_single_match() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(
            Inventory[(
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
            JF<Inventory, (&Name, JF<Weapons, (&Name, &Legendary)>)>,
        ), (With<Character>, JC<Inventory, JC<Weapons, With<Legendary>>>)>()
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
            Inventory[(
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
        .query_filtered::<(&Name, JF<Inventory, (&Name, J<Weapons, &Name>)>), (
            With<Character>,
            JC<Inventory, JC<Weapons, With<Legendary>>>,
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

#[test]
fn joined_empty_should_skip_and_yield_valid() {
    // Arrange
    let mut world = World::new();
    let valid = world.spawn(Name::new(VALID_NAME)).id();
    let invalid = world.spawn_empty().id();
    world.spawn(Inventory(vec![valid, invalid]));

    // Act
    let res: Vec<Vec<&Name>> = world.query::<J<Inventory, &Name>>().iter(&world).collect();

    // Assert
    assert_eq!(res[0].len(), 1);
    assert_eq!(res[0][0].as_str(), VALID_NAME);
}

#[test]
fn joined_with_despawned_target_should_skip() {
    // Arrange
    let mut world = World::new();
    let e = world.spawn(Name::new(GHOST_NAME)).id();
    world.spawn(Inventory(vec![e]));
    world.despawn(e);

    // Act
    let res: Vec<Vec<&Name>> = world.query::<J<Inventory, &Name>>().iter(&world).collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert!(res[0].is_empty());
}

#[test]
fn joined_children_should_yield_all() {
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
    let joined_names: Vec<&str> = res[0].iter().map(|n| n.as_str()).collect();
    assert!(joined_names.contains(&CHILD_1_NAME));
    assert!(joined_names.contains(&CHILD_2_NAME));
}

#[test]
fn joined_should_yield_empty() {
    // Arrange
    let mut world = World::new();
    world.spawn(Inventory(vec![]));

    // Act
    let res: Vec<_> = world.query::<J<Inventory, &Name>>().iter(&world).collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert!(res[0].is_empty());
}

#[test]
fn joined_first_should_filter() {
    // Arrange
    let mut world = World::new();
    world.spawn((Character, related!(Inventory[Name::new(ITEM_NAME)])));
    world.spawn((Character, Name::new(ENEMY_NAME)));

    // Act
    let res: Vec<&Name> = world
        .query_filtered::<(&Name, JF<Inventory, &Name>), ()>()
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
    world.spawn((Character, related!(Inventory[Name::new(ITEM_NAME)])));
    world.spawn((
        Character,
        Name::new(ENEMY_NAME),
        related!(Inventory[(Name::new(LEGENDARY_NAME), Legendary)]),
    ));

    // Act
    let res: Vec<&Name> = world
        .query::<(&Name, JF<Inventory, &Legendary>)>()
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
    world.spawn((Name::new(UNARMED_NAME), Inventory(vec![])));

    // Act
    let res: Vec<_> = world.query::<JF<Inventory, &Name>>().iter(&world).collect();

    // Assert
    assert!(res.is_empty());
}

#[test]
fn joined_first_deeply_nested_filter_condition_should_return() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(
            Inventory[(
                Name::new(CONTAINER_NAME),
                related!(Weapons[(Name::new(LEGENDARY_NAME), Legendary)])
            )]
        ),
    ));

    // Act
    let res: Vec<_> = world
        .query_filtered::<(
            &Name,
            JF<Inventory, (&Name, JF<Weapons, &Name>)>,
        ), (
            With<Character>,
            JC<Inventory, JC<Weapons, With<Legendary>>>,
        )>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    let (player_name, (container_name, item_name)) = res[0];
    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(container_name.as_str(), CONTAINER_NAME);
    assert_eq!(item_name.as_str(), LEGENDARY_NAME);
}

#[test]
fn join_condition_should_continue_searching() {
    // Arrange
    let mut world = World::new();
    world.spawn((
        Character,
        related!(Inventory[
            (Name::new(ITEM_NAME)),
            (Name::new(LEGENDARY_NAME), Legendary),
        ]),
    ));

    // Act
    let res: Vec<Entity> = world
        .query_filtered::<Entity, (With<Character>, JC<Inventory, With<Legendary>>)>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
}

#[test]
fn join_condition_empty_should_yield_nothing() {
    // Arrange
    let mut world = World::new();
    world.spawn((Name::new(EMPTY_NAME), Inventory(vec![])));

    // Act
    let res: Vec<Entity> = world
        .query_filtered::<Entity, JC<Inventory, With<Name>>>()
        .iter(&world)
        .collect();

    // Assert
    assert!(res.is_empty());
}
