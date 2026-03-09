# bevy_spliff
[![License](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/NicoZweifel/bevy_spliff?tab=readme-ov-file#licensecreditsinspirationsreferences)
[![Crates.io](https://img.shields.io/crates/v/bevy_spliff.svg)](https://crates.io/crates/bevy_spliff)

A crate for doing joins in bevy.

> [!CAUTION]
> This is an experiment.

## Setup

Just add the dependency.

```bash
cargo add bevy_spliff
```

And the `Joinable` derive macro:

```rust
#[derive(Component, Joinable)]
pub struct Weapons(pub Vec<Entity>);

#[derive(Component, Joinable)]
pub struct Target(pub Entity);
```

### Usage

Imagine you are writing a system that needs to fetch nested conditional data, 
currently this would look sth like this:

```rust
fn manual_system(
    q_characters: Query<(&Name, &Weapons), With<Character>>,
    q_weapons: Query<&Name, With<Legendary>>,
) {
    for (name, weapons) in &q_characters {
        let weapon_names: Vec<&Name> = weapons.0.iter()
            .filter_map(|&e| q_weapons.get(e).ok())
            .collect();

        println!("Character {} has legendary weapons: {:?}", name, weapon_names);
    }
}
```

Using `bevy_spliff` this simplifies to:

```rust
fn joined_system(
    q: Query<
        (&Name, Joined<Weapons, &Name>),
        (With<Character>, JoinCondition<Weapons, With<Legendary>>),
    >,
) {
    for (name, weapon_names) in &q {
        println!(
            "Character {} has legendary weapons: {:?}",
            name, weapon_names
        );
    }
}
```

or just this if you don't need to use a nested filter condition:

```rust
fn simple_joined_system(
    q: Query<
        (&Name, Joined<Weapons, &Name>),
        With<Character>,
    >,
) {
    for (name, weapon_names) in &q {
        println!(
            "Character {} has weapons: {:?}",
            name, weapon_names
        );
    }
}
```

You can use the `type-aliases` feature, which is enabled by default, if you prefer even shorter syntax:

```rust
fn aliased_joined_system(
    q: Query<(&Name, J<Weapons, &Name>), (With<Character>, JC<Weapons, With<Legendary>>)>,
) {
    for (name, weapon_names) in &q {
        println!(
            "Character {} has legendary weapons: {:?}",
            name, weapon_names
        );
    }
}
```

This also works for deeply nested relational queries:

```rust
fn deeply_nested_joined_system(
    q: Query<
        (&Name, J<Armors, (&Name, J<Weapons, &Name>)>),
        (With<Character>, JC<Armors, JC<Weapons, With<Legendary>>>),
    >,
) {
    for (name, armors) in &q {
        for (armor_name, weapon_names) in armors {
            println!(
                "Character {} with armor {:?} has legendary weapons: {:?}",
                name, armor_name, weapon_names
            );
        }
    }
}
```

You can also use `JoinedFirst` or `JF` to inner join on the first match.

```rust
fn deeply_nested_joined_first_system(
    q: Query<
        (&Name, JF<Armors, (&Name, JF<Weapons, &Name>)>),
        (With<Character>, JC<Armors, JC<Weapons, With<Legendary>>>),
    >,
) {
    for (name, (armor_name, weapon_name)) in &q {
        println!(
            "Character {} with armor {:?} has legendary weapons: {:?}",
            name, armor_name, weapon_name
        );
    }
}
```

For complex systems, you can use J, JF, and JC inside structs that derive QueryData or QueryFilter.
This allows you to provide descriptive names for your joined data:

```rust
#[derive(QueryData)]
pub struct CharacterWeaponQueryData {
    name: &'static Name,
    weapon_names: J<Weapons, &'static Name>,
}

#[derive(QueryFilter)]
pub struct CharacterWeaponFilter {
    _is_character: With<Character>,
    _has_legendary: JC<Weapons, With<Legendary>>,
}

fn complex_joined_system(query: Query<CharacterWeaponQueryData, CharacterWeaponFilter>) {
    for character in &query {
        println!(
            "Character {} has legendary weapons: {:?}", 
            character.name, 
            character.weapon_names
        );
    }
}
```

## Overview

| Feature              | Type Alias        | Description | Example Usage |
|----------------------|-------------------| --- | --- |
| **Joined**           | `J<Ref, Data>`    | Fetches data from all valid targets. Returns `Vec` or `Option` based on the mapper. | `J<Weapons, &Name>` |
| **Joined First**     | `JF<Ref, Data>`   | Traverses a relationship and returns only the first target that matches the query data. | `JF<Armors, &Name>` |
| **Join Condition**   | `JC<Ref, Filter>` | A query filter that checks if any target of a relationship satisfies a specific condition. | `JC<Weapons, With<Legendary>>` |
| **Derive Macro**     | `Joinable`        | Automatically implements the `Joinable` trait for structs containing an `Entity` or `Vec<Entity>`. | `#[derive(Joinable)]` |
| **Deep Nesting**     | N/A               | Supports recursive joins (joining on a joined result) for complex hierarchy traversals. | `J<A, (Data, J<B, Data>)>` |
| **Built-in Support** | N/A               | Native support for standard Bevy hierarchy components like `Children` and `ChildOf`. | `J<Children, &Name>` |
| **Change Detection** | N/A               | Integrates with Bevy's change detection within the join filters. | `JC<R, Changed<T>>` |

### Key Definitions

* **`R` (Relationship)**: A component implementing `Joinable` (e.g., a field containing a target `Entity`).
* **`D` (Data)**: The `QueryData` you wish to retrieve from the target entity.
* **`F` (Filter)**: A `QueryFilter` to validate the target entity without fetching data.

## SQL Analogs

If you are coming from a relational database background, here is how `bevy_spliff` types conceptually map to SQL operations. 

Because Bevy queries do not duplicate the "Root" entity for multiple matches (unlike standard SQL joins), `bevy_spliff` uses a combination of Data and Filters to achieve relational results:

| `bevy_spliff`              | SQL Equivalent                                            | Behavior | Empty/Broken List Behavior |
|:---------------------------|:----------------------------------------------------------| :--- | :--- |
| **`J<Ref, Data>`** | `LEFT JOIN target WHERE target.Data IS NOT NULL`          | Fetches targets that have `D`. | Keeps the root entity, returns an empty `Vec`. |
| **`J<Ref, Option<Data>>`** | `LEFT JOIN target`                                        | Fetches all targets, wrapping data in `Option`. | Keeps the root entity, returns an empty `Vec`. |
| **`JF<Ref, Data>`** | `INNER JOIN target WHERE target.Data IS NOT NULL` | Fetches the first target that has `D`. | Filters out the root entity from the query. |
| **`JC<Ref, Filter>`** | `WHERE EXISTS (SELECT 1 FROM target WHERE F)`             | Strict filter condition on the entire row without fetching data. | Filters out the root entity from the query. |
| **`J` + `JC`** | `INNER JOIN target` (1-to-Many)                           | Fetches all matches as a `Vec`, strictly requires at least one target to pass `JC`. | Filters out the root entity from the query. |

> [!TIP]
> For a standard Inner Join, **`JF`** is usually your best option. However, if you need a **1-to-Many Inner Join** (fetching all matching targets but skipping root entities that have zero matches), combine `J<Ref, Data>` in your query data with `JC<Ref, Filter>` in your query filter.

## Okay... so when to use what?

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
- `derive`: Enables the #[derive(Joinable)] macro.

## Notes / TODOs

- Depends on `bevy_ecs` only.
- Mutable access on `Joined` would be great, rn it's `ReadOnlyQueryData` for `Joined`.
- More test cases and organizing suites.
- Make sure this doesn't do something terribly wrong.
- If you cannot guarantee a stable order in your data but need to handle all potential matches, 
it is safer to use Joined (J) to fetch a Vec of all matches and then apply your own sorting logic inside the system body. 
- docs
- more/better tests

## License

The code is dual-licensed:

* MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))


