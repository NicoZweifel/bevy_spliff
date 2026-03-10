# bevy_spliff 💨
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/NicoZweifel/bevy_spliff?tab=readme-ov-file#licensecreditsinspirationsreferences)
[![Crates.io](https://img.shields.io/crates/v/bevy_spliff.svg)](https://crates.io/crates/bevy_spliff)
[![Downloads](https://img.shields.io/crates/d/bevy_spliff.svg)](https://crates.io/crates/bevy_spliff)
[![Docs](https://docs.rs/bevy_spliff/badge.svg)](https://docs.rs/bevy_spliff/)
[![CI](https://github.com/NicoZweifel/bevy_spliff/actions/workflows/ci.yaml/badge.svg?branch=dev)](https://github.com/NicoZweifel/bevy_spliff/actions/workflows/ci.yaml)

A crate for doing joins in bevy.

> [!CAUTION]
> This is an experiment.

## Setup

Just add the dependency.

```bash
cargo add bevy_spliff
```

And the `Joinable` derive macro to your relations, e.g.:

```rust
#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = InventoryItemOf)]
struct InventoryItems(Vec<Entity>);

#[derive(Component, Joinable)]
#[relationship(relationship_target = InventoryItems)]
struct InventoryItemOf(pub Entity);

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = WeaponOf)]
struct Weapons(Vec<Entity>);

#[derive(Component, Joinable)]
#[relationship(relationship_target = Weapons)]
struct WeaponOf(pub Entity);

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = StorageItemOf)]
struct StorageItems(Vec<Entity>);

#[derive(Component, Joinable)]
#[relationship(relationship_target = StorageItems)]
struct StorageItemOf(pub Entity);
```

### Usage

Imagine you are writing a system that needs to fetch nested conditional data, 
currently this would look sth like this:

```rust
fn manual_system(
    q_characters: Query<(&Name, &InventoryItems), With<Character>>,
    q_items: Query<&Name, With<Legendary>>,
) {
    for (name, inventory) in &q_characters {
        let item_names: Vec<&Name> = inventory.0.iter()
            .filter_map(|&e| q_items.get(e).ok())
            .collect();

        println!("Character {:?} has legendary items: {:?}", name, item_names);
    }
}
```

This simplifies to:

```rust
fn joined_system(
    q: Query<
        (&Name, Joined<InventoryItems, &Name>),
        (With<Character>, JoinCondition<InventoryItems, With<Legendary>>),
    >,
) {
    for (name, item_names) in &q {
        println!("Character {:?} has legendary items: {:?}", name, item_names);
    }
}
```

Or just this if you don't need to use a nested filter condition:

```rust
fn simple_joined_system(
    q: Query<
        (&Name, Joined<InventoryItems, &Name>),
        With<Character>,
    >,
) {
    for (name, item_names) in &q {
        println!(
            "Character {:?} has items: {:?}",
            name, item_names 
        );
    }
}
```

You can use the `type-aliases` feature, which is enabled by default, if you prefer the short syntax:

```rust
fn aliased_joined_system(
    q: Query<(&Name, J<Weapons, &Name>), (With<Character>, JC<Weapons, With<Legendary>>)>,
) {
    for (name, item_names) in &q {
        println!("Character {:?} has legendary weapons: {:?}", name, item_names);
    }
}
```

This also works for deeply nested relational queries, e.g. the entity has a "Vault" containing a "Backpack" with legendary weapons:

```rust
fn deeply_nested_joined_system(
    q: Query<
        (
            &Name,
            J<
                StorageItems,
                (
                    &Name,
                    J<InventoryItems, (&Name, J<Weapons, (&Name, &Legendary)>)>,
                ),
            >,
        ),
        (
            With<Character>,
            JC<StorageItems, JC<InventoryItems, JC<Weapons, With<Legendary>>>>,
        ),
    >,
) {
    for (character_name, storages) in &q {
        for (storage_name, inventories) in storages {
            for (inventory_name, weapons) in inventories {
                let weapon_names: Vec<&Name> = weapons.iter().map(|(n, _)| n).collect();
                println!(
                    "{:?} has a {:?} containing a {:?} with legendary weapons: {:?}",
                    character_name, storage_name, inventory_name, weapon_names
                );
            }
        }
    }
}
```

You can also use `JoinedFirst` or `JF` to inner join on the first match, e.g.,
each entity has a "Vault" containing a "Backpack" and the first legendary weapon found it finds.

```rust
fn deeply_nested_joined_first_system(
    q: Query<
        (
            &Name,
            JF<
                StorageItems,
                (
                    &Name,
                    JF<InventoryItems, (&Name, JF<Weapons, (&Name, &Legendary)>)>,
                ),
            >,
        ),
        (
            With<Character>,
            JC<StorageItems, JC<InventoryItems, JC<Weapons, With<Legendary>>>>,
        ),
    >,
) {
    for (character_name, (storage_name, (inventory_name, (weapon_name, _)))) in &q {
        println!(
            "{:?} has a {:?} containing a {:?} with legendary weapon: {:?}",
            character_name, storage_name, inventory_name, weapon_name
        );
    }
}
```

For complex systems with deeply nested queries, you can use `J`, `JF`, and `JC` inside structs that derive QueryData or QueryFilter. 
This might resolve warnings/readability issues and allows you to provide descriptive names for your joined data:

```rust
#[derive(QueryData)]
pub struct CharacterItemQueryData {
    name: &'static Name,
    items: J<InventoryItems, &'static Name>,
}

#[derive(QueryFilter)]
pub struct CharacterItemFilter {
    _is_character: With<Character>,
    _has_legendary: JC<InventoryItems, With<Legendary>>,
}

fn complex_joined_system(query: Query<CharacterItemQueryData, CharacterItemFilter>) {
    for character in &query {
        println!(
            "Character {:?} has legendary items: {:?}", 
            character.name, 
            character.items
        );
    }
}
```

## Overview

| Feature              | Type Alias        | Description                                                                                                                                                             | Example Usage                  |
|----------------------|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------|
| **Joined**           | `J<Ref, Data>`    | Fetches a `Vec` or `Option` containing targets matching Data. If combined with `JC`, it eagerly fetches the full list as long as at least one target passes the filter. | `J<Weapons, &Name>`            |
| **Joined First**     | `JF<Ref, Data>`   | Traverses a relationship and returns only the first target that matches the query data.                                                                                 | `JF<Weapons, &Name>`           |
| **Join Condition**   | `JC<Ref, Filter>` | A query filter that checks if any target of a relationship satisfies a specific condition.                                                                              | `JC<Weapons, With<Legendary>>` |
| **Derive Macro**     | `Joinable`        | Automatically implements the `Joinable` trait for structs containing an `Entity` or `Vec<Entity>`.                                                                      | `#[derive(Joinable)]`          |
| **Deep Nesting**     | N/A               | Supports recursive joins (joining on a joined result) for complex hierarchy traversals.                                                                                 | `J<A, (Data, J<B, Data>)>`     |
| **Built-in Support** | N/A               | Native support for standard Bevy hierarchy components like `Children` and `ChildOf`.                                                                                    | `J<Children, &Name>`           |
| **Change Detection** | N/A               | Integrates with Bevy's change detection within the join filters.                                                                                                        | `JC<R, Changed<T>>`            |

### Key Definitions

* `Ref`: A relational component implementing `Joinable` (e.g., a field containing a target `Entity`).
* `Data`: The `QueryData` you wish to retrieve from the target entity.
* `Filter`: A `QueryFilter` to validate the target entity without fetching data.

## SQL Analogs

If you are coming from a relational database background, here is how `bevy_spliff` types conceptually map to standard SQL operations. 

Because Bevy queries do not duplicate the "Root" entity for multiple matches (unlike standard SQL joins, which return multiple rows), `bevy_spliff` instead aggregates the targets into a `Vec`.

| `bevy_spliff`                           | SQL Concept                     | Behavior                                                                                  | Empty/Broken List Behavior |
|:----------------------------------------|:--------------------------------|:------------------------------------------------------------------------------------------| :--- |
| `J<Ref, Data>`                          | `LEFT JOIN`                     | Fetches an array of targets that possess the requested `Data`.                            | Keeps the root entity, returns an empty `Vec`. |
| `J<Ref, Option<Data>>`                  | `LEFT JOIN`                     | Fetches an array of all targets, wrapping missing data in `None`.                         | Keeps the root entity, returns an empty `Vec`. |
| `JF<Ref, Data>`                         | `INNER JOIN` (First Match)      | Fetches only the first target that possesses the requested `Data`.                        | Filters out the root entity from the query. |
| `JC<Ref, Filter>`                       | `WHERE EXISTS`                  | Evaluates a condition against targets without fetching their data.                        | Filters out the root entity from the query. |
| `J<Ref, Data>` + `JC<Ref, Filter>`      | `LEFT JOIN` + `WHERE EXISTS`    | Fetches **all** matches as a `Vec`, requires $\ge 1$ match but only applies `JC` to root. | Filters out the root entity from the query. |
| `J<Ref, (Data, &C)>` + `JC<Ref, With<C>>`| `INNER JOIN` (1-to-Many)        | Fetches a `Vec` of strictly matching targets, and requires $\ge 1$ match.                 | Filters out the root entity from the query. |

> [!TIP]
> **Understanding 1-to-Many Joins:** 
> By default, combining `J` with `JC` acts as an existence check: if *any* target passes `JC`, 
> the `J` fetcher eagerly fetches **all** targets matching its `QueryData`.
> 
> To create a strict 1-to-Many Inner Join (where the resulting Vec only contains specific targets, 
> AND the root entity is skipped if there are zero matches), 
> require the component in your Data tuple and include JC in your query filter:
> `Query<(&Name, J<Weapons, (&Name, &Legendary)>), JC<Weapons, With<Legendary>>>`
> 
> You can also use `Has` to filter in the system body, if desired.

## Okay... so when to use what then? 

Choosing between `J`, `JF`, and `JC` comes down to **Optionality** (do they *need* to have it?) and **Multiplicity** (do you need *all* of them, or just *one*?).

* **I need to process all related items.**
  -> Use **`J`** (Returns a `Vec`). 
  *Example: Calculating the total weight of a player's inventory.*
* **I need one related item, but it's okay if they don't have any.**
  -> Use **`J`** (Returns an empty `Vec` or `Option` depending on the mapper).
  *Example: Drawing a weapon icon on the UI. Unarmed players should still have their UI drawn, just with an empty weapon slot.*
* **I need one related item, and the system should SKIP entities that don't have it.**
  -> Use **`JF`** (Returns the item directly, acts as an Inner Join).
  *Example: A combat system. Players without an equipped weapon cannot attack and should be skipped by the system.*
* **I don't need the data, I just care IF they have it.** Use **`JC`** (Acts as a Query Filter).
  *Example: A healing system or for usage with marker components, e.g. you only want to query players that have a Health Potion in their inventory, but you don't need to read the data yet.*

## Feature Flags

These are enabled by default.

- `type-aliases`: Enables shorthand names like J, JF, and JC.
- `derive`: Enables the `#[derive(Joinable)]` macro.

## Notes / TODOs

- Mutable access on `Joined` would be great, rn it's `ReadOnlyQueryData` for `Joined`.
- Use [`NestedQuery`](https://github.com/bevyengine/bevy/pull/21557), which should also solve mutability for `Joined`.
- Cleanup `WorldQuery`/`QueryData` implementations with e.g. generic `JoinQuery` implementations.
- Depends on `bevy_ecs` only.
- More test cases and organizing suites.
- Make sure this doesn't do something terribly wrong.
- Bevy's iteration order is nondeterministic. if you need stable sorting,
use Joined (J) to get a Vec and sort it in the system body.
- docs
- more/better tests

## License

The code is dual-licensed:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))


