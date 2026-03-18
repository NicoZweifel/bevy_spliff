use super::constants::*;
use bevy_ecs::{name::Name, prelude::*};
use bevy_spliff::prelude::*;

#[derive(Component)]
#[require(InventoryItems, StorageItems, Name::new(PLAYER_NAME))]
pub struct Character;

#[derive(Component)]
pub struct Legendary;

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = StorageItemOf)]
pub struct StorageItems(Vec<Entity>);

#[derive(Component, Joinable)]
#[relationship(relationship_target = StorageItems)]
pub struct StorageItemOf(pub Entity);

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = InventoryItemOf)]
pub struct InventoryItems(Vec<Entity>);

impl InventoryItems {
    pub fn new(items: Vec<Entity>) -> Self {
        Self(items)
    }
}

#[derive(Component, Joinable)]
#[relationship(relationship_target = InventoryItems)]
pub struct InventoryItemOf(pub Entity);

#[derive(Component, Joinable, Default)]
#[relationship_target(relationship = WeaponOf)]
pub struct Weapons(Vec<Entity>);

#[derive(Component, Joinable)]
#[relationship(relationship_target = Weapons)]
pub struct WeaponOf(pub Entity);
