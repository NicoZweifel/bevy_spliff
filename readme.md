# bevy_spliff

A crate for doing joins in bevy.

> [!CAUTION]
> This is an experiment.

## Setup

Just add the dependency.

```bash
cargo add bevy_spliff
```

### Usage

Imagine you are writing a system that needs to fetch nested conditional data, 
currently this would look sth like this:

```rust
fn manual_system(
    q_characters: Query<(&Name, &Weapons), With<Character>>,
    q_weapons: Query<(&Name, &Legendary)>,
) {
    for (name, weapons) in &q_characters {
        let weapon_names: Vec<&str> = weapons.0.iter()
            .filter_map(|&e| q_weapons.get(e).ok())
            .map(|(name, _)| name.0.as_str())
            .collect();
        
        println!("Character {} has legendary weapons: {:?}", name.0, weapon_names);
    }
}
```

Using `bevy_spliff` this simplifies to:

```rust
fn spliff_system(
    q: Query<
        (&Name, Joined<Weapons, &Name>),
        (With<Character>, JoinCondition<Weapons, With<Legendary>>),
    >,
) {
    for (name, weapon_names) in &query {
        println!(
            "Character {} has legendary weapons: {:?}",
            name.0, weapon_names
        );
    }
}
```

You can use the `type-aliases` feature, which is enabled by default, if you prefer even shorter syntax:

```rust
fn aliased_spliff_system(
    q: Query<(&Name, J<Weapons, &Name>), (With<Character>, JC<Weapons, With<Legendary>>)>,
) {
    for (name, weapon_names) in &query {
        println!(
            "Character {} has legendary weapons: {:?}",
            name.0, weapon_names
        );
    }
}
```

When managing very complex types you can declare structs and derive `QueryData`, just as you would with complex queries in general:

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

fn spliff_system(query: Query<CharacterWeaponQueryData, CharacterWeaponFilter>) {
    for character in &query {
        println!(
            "Character {} has legendary weapons: {:?}", 
            character.name.0, 
            character.weapon_names
        );
    }
}
```

## Overview

| Feature              | Type Alias | Description | Example Usage |
|----------------------| --- | --- | --- |
| **Joined**           | `J<R, D>` | Fetches data from all valid targets. Returns `Vec` or `Option` based on the mapper. | `J<Weapons, &Name>` |
| **Joined First**     | `JF<R, D>` | Traverses a relationship and returns only the first target that matches the query data. | `JS<Armors, &Name>` |
| **Join Condition**   | `JC<R, F>` | A query filter that checks if any target of a relationship satisfies a specific condition. | `JF<Weapons, With<Legendary>>` |
| **Derive Macro**     | `Joinable` | Automatically implements the `Joinable` trait for structs containing an `Entity` or `Vec<Entity>`. | `#[derive(Joinable)]` |
| **Deep Nesting**     | N/A | Supports recursive joins (joining on a joined result) for complex hierarchy traversals. | `JF<A, (Data, JF<B, Data>)>` |
| **Built-in Support** | N/A | Native support for standard Bevy hierarchy components like `Children` and `ChildOf`. | `J<Children, &Name>` |
| **Change Detection** | N/A | Integrates with Bevy's change detection within the join filters. | `JC<R, Changed<T>>` |

### Key Definitions

* **`R` (Relationship)**: A component implementing `Joinable` (e.g., a field containing a target `Entity`).
* **`D` (Data)**: The `QueryData` you wish to retrieve from the target entity.
* **`F` (Filter)**: A `QueryFilter` to validate the target entity without fetching data.

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

