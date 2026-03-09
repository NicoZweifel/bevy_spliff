use bevy_ecs::{lifecycle::HookContext, name::Name, prelude::*, world::DeferredWorld};
use bevy_spliff::prelude::*;

const PLAYER_NAME: &str = "Player";
const ENEMY_NAME: &str = "Enemy";

#[derive(Component)]
#[require(Armors, Weapons, Name::new(PLAYER_NAME))]
struct Character;

#[derive(Component)]
struct Legendary;

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = ArmorOf)]
struct Armors(Vec<Entity>);

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = WeaponOf)]
struct Weapons(Vec<Entity>);

#[derive(Component, Joinable, Clone)]
#[component(on_add = Self::on_add)]
#[relationship(relationship_target = Weapons)]
#[require(Name::new("Weapon"))]
struct WeaponOf(Entity);

impl WeaponOf {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let mut weapons = world
            .get::<WeaponOf>(ctx.entity)
            .cloned()
            .and_then(|weapon| world.get_mut::<Weapons>(weapon.0))
            .unwrap();

        if !weapons.0.contains(&ctx.entity) {
            weapons.0.push(ctx.entity);
        }
    }
}

/// For the purpose of testing deeply nested queries `Weapons` can be related to `ArmorOf` parts as well.
///
/// E.g. a weapon in a pocket.
#[derive(Component, Joinable, Clone)]
#[component(on_add = Self::on_add)]
#[relationship(relationship_target = Armors)]
#[require(Name::new("Armor"), Weapons)]
struct ArmorOf(Entity);

impl ArmorOf {
    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let mut weapons = world
            .get::<ArmorOf>(ctx.entity)
            .cloned()
            .and_then(|armor| world.get_mut::<Armors>(armor.0))
            .unwrap();

        if !weapons.0.contains(&ctx.entity) {
            weapons.0.push(ctx.entity);
        }
    }
}

#[test]
fn joined_one_to_many_should_yield_all() {
    // Arrange
    let mut world = World::new();
    let hero_id = world.spawn(Character).id();
    world.spawn((ChildOf(hero_id), WeaponOf(hero_id)));
    world.spawn((ChildOf(hero_id), WeaponOf(hero_id), Name::new("Knife")));
    world.spawn((ChildOf(hero_id), ArmorOf(hero_id)));
    world.spawn((ChildOf(hero_id), ArmorOf(hero_id), Name::new("Helmet")));

    // Act
    let results: Vec<_> = world
        .query_filtered::<(&Name, J<Weapons, &Name>, J<Armors, &Name>), With<Character>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(results.len(), 1);
    let (player_name, weapon_names, armor_names) = &results[0];
    let weapon_names: Vec<&str> = weapon_names.iter().map(|n| n.as_str()).collect();
    let armor_names: Vec<&str> = armor_names.iter().map(|n| n.as_str()).collect();
    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert!(weapon_names.contains(&"Weapon"));
    assert!(weapon_names.contains(&"Knife"));
    assert!(armor_names.contains(&"Armor"));
    assert!(armor_names.contains(&"Helmet"));
}

#[test]
fn joined_deeply_nested_filtered_should_yield_all() {
    // Arrange
    let mut world = World::new();

    let e1 = world.spawn(Character).id();
    let e2 = world.spawn(ArmorOf(e1)).id();
    world.spawn((WeaponOf(e2), Legendary, Name::new("Magic Sword")));
    world.spawn((WeaponOf(e2), Legendary, Name::new("Magic Shield")));

    // Act
    let res: Vec<_> = world
        .query_filtered::<(&Name, J<Armors, (&Name, J<Weapons, &Name>)>), (
            With<Character>,
            JC<Armors, JC<Weapons, With<Legendary>>>,
        )>()
        .iter(&world)
        .collect();

    // Assert
    let (player_name, armor_names) = &res[0];
    let (armor_name, weapon_names) = &armor_names[0];
    let weapon_names: Vec<&str> = weapon_names.iter().map(|n| n.as_str()).collect();

    assert_eq!(res.len(), 1);
    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(armor_name.as_str(), "Armor");
    assert!(weapon_names.contains(&"Magic Sword"));
    assert!(weapon_names.contains(&"Magic Shield"));
}

/* take care of this later
#[test]
fn joined_first_empty_should_() {
    // Arrange
    let mut world = World::new();
    let valid = world.spawn(Name::new("Valid".into())).id();
    // TODO handle this gracefully if possible?
    let invalid = world.spawn_empty().id();

    world.spawn(Weapons(vec![valid, invalid]));

    // Act
    let res: Vec<_> = world
        .query::<JoinedFirst<Weapons, &Name>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
}
 */

#[test]
fn joined_empty_should_skip_and_yield_valid() {
    // Arrange
    let mut world = World::new();
    let valid = world.spawn(Name::new("Valid")).id();
    let invalid = world.spawn_empty().id();

    world.spawn(Weapons(vec![valid, invalid]));

    // Act
    let res: Vec<Vec<&Name>> = world.query::<J<Weapons, &Name>>().iter(&world).collect();

    // Assert
    assert_eq!(res[0].len(), 1);
    assert_eq!(res[0][0].as_str(), "Valid");
}

#[test]
fn joined_with_despawned_target_should_skip() {
    // Arrange
    let mut world = World::new();

    let e = world.spawn(Name::new("Ghost")).id();
    world.spawn(Weapons(vec![e]));
    world.despawn(e);

    // Act
    let res: Vec<Vec<&Name>> = world.query::<J<Weapons, &Name>>().iter(&world).collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert!(res[0].is_empty());
}

#[test]
fn joined_children_should_yield_all() {
    // Arrange
    let mut world = World::new();

    let parent = world.spawn(Name::new("Parent")).id();

    world.spawn((Name::new("Child 1"), ChildOf(parent)));
    world.spawn((Name::new("Child 2"), ChildOf(parent)));

    // Act
    let res: Vec<Vec<&Name>> = world
        .query_filtered::<J<Children, &Name>, With<Children>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].len(), 2);
    let joined_names: Vec<&str> = res[0].iter().map(|n| n.as_str()).collect();
    assert!(joined_names.contains(&"Child 1"));
    assert!(joined_names.contains(&"Child 2"));
}

#[test]
fn joined_should_yield_empty() {
    // Arrange
    let mut world = World::new();

    world.spawn(Weapons(vec![]));

    // Act
    let res: Vec<_> = world.query::<J<Weapons, &Name>>().iter(&world).collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert!(res[0].is_empty());
}

#[test]
fn joined_resilience_to_despawned_targets() {
    // Arrange
    let mut world = World::new();

    let player = world.spawn(Character).id();
    let sword = world.spawn((WeaponOf(player), Legendary)).id();
    let mut query = world.query_filtered::<J<Weapons, Entity>, With<Character>>();

    // Act
    let init_res = query.single(&world).unwrap();
    world.despawn(sword);
    let res = query.single(&world).unwrap();

    // Assert
    assert!(init_res.contains(&sword));
    assert!(!res.contains(&sword));
}

#[test]
fn joined_first_mapper_should_return_option() {
    // Arrange
    let mut world = World::new();

    let player = world.spawn((Character, Name::new("Hero"))).id();
    world.spawn(WeaponOf(player));

    // Act
    let res = world.query::<J<WeaponOf, &Name>>().single(&world);

    // Assert
    assert!(res.is_ok());
    assert_eq!(res.unwrap().unwrap().as_str(), "Hero");
}

#[test]
fn joined_first_should_filter() {
    // Arrange
    let mut world = World::new();

    let e = world.spawn(Character).id();
    world.spawn((WeaponOf(e), ChildOf(e)));
    world.spawn((Character, Name::new(ENEMY_NAME)));

    // Act
    let res: Vec<&Name> = world
        .query_filtered::<(&Name, JF<Weapons, &Name>), ()>()
        .iter(&world)
        .map(|x| x.0)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].as_str(), PLAYER_NAME);
}

#[test]
fn joined_first_component_should_filter() {
    // Assert
    let mut world = World::new();

    let e1 = world.spawn(Character).id();
    world.spawn((WeaponOf(e1), ChildOf(e1)));

    let e2 = world.spawn((Character, Name::new(ENEMY_NAME))).id();
    world.spawn((WeaponOf(e2), ChildOf(e2), Legendary));

    // Act
    let res: Vec<&Name> = world
        .query::<(&Name, JF<Weapons, &Legendary>)>()
        .iter(&world)
        .map(|x| x.0)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].as_str(), ENEMY_NAME);
}

#[test]
fn joined_first_filtered_should_filter() {
    // Arrange
    let mut world = World::new();

    let e1 = world.spawn(Character).id();
    world.spawn((WeaponOf(e1), ChildOf(e1)));

    let e2 = world.spawn((Character, Name::new(ENEMY_NAME))).id();
    world.spawn((WeaponOf(e2), ChildOf(e2), Legendary));

    // Act
    let res: Vec<&Name> = world
        .query_filtered::<(&Name, JF<Weapons, &Name>), JC<Weapons, With<Legendary>>>()
        .iter(&world)
        .map(|x| x.0)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0].as_str(), ENEMY_NAME);
}

#[test]
fn joined_first_with_despawned_target_should_skip() {
    // Arrange
    let mut world = World::new();
    let target = world.spawn(Name::new("Ghost")).id();
    world.spawn(Weapons(vec![target]));
    world.despawn(target);

    // Act
    let res: Vec<_> = world
        .query::<(Entity, JF<Weapons, &Name>)>()
        .iter(&world)
        .collect();

    // Assert
    assert!(res.is_empty());
}

#[test]
fn joined_first_should_return_empty() {
    // Arrange
    let mut world = World::new();

    let target = world.spawn(Armors(vec![])).id();
    world.spawn(Weapons(vec![target]));

    // Act
    let res: Vec<_> = world
        .query::<(Entity, JF<Weapons, JF<Armors, &Name>>)>()
        .iter(&world)
        .collect();

    // Assert
    assert!(res.is_empty());
}

#[test]
fn joined_first_empty_should_filter_out_root() {
    // Arrange
    let mut world = World::new();

    world.spawn((Name::new("Unarmed"), Weapons(vec![])));

    // Act
    let res: Vec<_> = world.query::<JF<Weapons, &Name>>().iter(&world).collect();

    // Assert
    assert!(res.is_empty());
}

#[test]
fn joined_first_deeply_nested_filtered_should_return() {
    // Arrange
    let mut world = World::new();

    let e1 = world.spawn(Character).id();
    let e2 = world.spawn(ArmorOf(e1)).id();
    world.spawn((WeaponOf(e2), Legendary, Name::new("Magic Sword")));

    // Act
    let res: Vec<_> = world
        .query_filtered::<(
            &Name,
            JF<Armors, (&Name, JF<Weapons, &Name>)>,
        ), (
            With<Character>,
            JC<Armors, JC<Weapons, With<Legendary>>>,
        )>()
        .iter(&world)
        .collect();

    // Assert
    let (player_name, (armor_name, weapon_name)) = res[0];
    assert_eq!(res.len(), 1);
    assert_eq!(player_name.as_str(), PLAYER_NAME);
    assert_eq!(armor_name.as_str(), "Armor");
    assert_eq!(weapon_name.as_str(), "Magic Sword");
}

#[test]
fn joined_first_should_skip_ghost_and_find_valid() {
    // Arrange
    let mut world = World::new();

    let ghost = world.spawn(Name::new("Ghost")).id();
    let valid = world.spawn(Name::new("Valid")).id();

    world.spawn(Weapons(vec![ghost, valid]));
    world.despawn(ghost);

    // Act
    let res = world.query::<JF<Weapons, &Name>>().single(&world).unwrap();

    // Assert
    assert_eq!(res.as_str(), "Valid");
}

#[test]
fn join_condition_target_should_fail_condition() {
    // Arrange
    let mut world = World::new();
    let target = world.spawn(Name::new("Target")).id();
    world.spawn(Weapons(vec![target]));
    world.clear_trackers();

    // Act
    let res: Vec<_> = world
        .query_filtered::<Entity, JC<Weapons, Changed<Name>>>()
        .iter(&world)
        .collect();

    // Assert
    assert!(res.is_empty());
}

#[test]
fn join_condition_should_continue_searching() {
    // Arrange
    let mut world = World::new();
    let player = world.spawn((Character, Name::new(PLAYER_NAME))).id();

    world.spawn(WeaponOf(player));
    world.spawn((WeaponOf(player), Legendary));

    // Act
    let res: Vec<Entity> = world
        .query_filtered::<Entity, (With<Character>, JC<Weapons, With<Legendary>>)>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], player);
}

#[test]
fn join_condition_should_detect_added_targets() {
    // Arrange
    let mut world = World::new();

    let e1 = world.spawn(Character).id();
    world.spawn((WeaponOf(e1), ChildOf(e1)));
    world.clear_trackers();

    let e2 = world.spawn(Character).id();
    world.spawn((WeaponOf(e2), ChildOf(e2)));

    // Act
    let res: Vec<Entity> = world
        .query_filtered::<Entity, JC<Weapons, Added<WeaponOf>>>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], e2);
}

#[test]
fn join_condition_should_be_true_if_any_target_is_valid() {
    // Arrange
    let mut world = World::new();
    let player = world.spawn((Character, Name::new(PLAYER_NAME))).id();

    let ghost = world.spawn(Name::new("Ghost")).id();
    let valid = world.spawn(Legendary).id();

    world.get_mut::<Weapons>(player).unwrap().0 = vec![ghost, valid];
    world.despawn(ghost);

    // Act
    let res: Vec<Entity> = world
        .query_filtered::<Entity, (With<Character>, JC<Weapons, With<Legendary>>)>()
        .iter(&world)
        .collect();

    // Assert
    assert_eq!(res.len(), 1);
    assert_eq!(res[0], player);
}

#[test]
fn join_condition_empty_should_yield_nothing() {
    // Arrange
    let mut world = World::new();

    world.spawn((Name::new("Empty"), Weapons(vec![])));

    // Act
    let res: Vec<Entity> = world
        .query_filtered::<Entity, JC<Weapons, With<Name>>>()
        .iter(&world)
        .collect();

    // Assert
    assert!(res.is_empty());
}
