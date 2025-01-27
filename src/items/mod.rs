//! Things available to spawn from the level editor
//! Proto-mods, eventually some of the items will move to some sort of a wasm runtime

use macroquad::{
    experimental::{
        collections::storage,
        scene::{HandleUntyped, Node, RefMut},
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    capabilities::{NetworkReplicate, PhysicsObject},
    components::{PhysicsBody, Sprite, SpriteParams},
    json, GameWorld,
};

mod weapons;
pub use weapons::{Weapon, WeaponAnimationParams, WeaponParams};

mod equipped;
pub use equipped::{EquippedItem, EquippedItemParams};

mod sproinger;
pub use sproinger::Sproinger;

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ItemKind {
    Weapon {
        #[serde(flatten)]
        params: WeaponParams,
    },
    EquippedItem {
        #[serde(flatten)]
        params: EquippedItemParams,
    },
}

impl ItemKind {
    pub fn is_weapon(&self) -> bool {
        if let Self::Weapon { .. } = self {
            return true;
        }

        false
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ItemParams {
    pub id: String,
    #[serde(flatten)]
    pub kind: ItemKind,
    pub sprite: SpriteParams,
    #[serde(with = "json::uvec2_def")]
    pub collider_size: UVec2,
    #[serde(default)]
    pub is_network_ready: bool,
}

pub struct Item {
    pub id: String,
    pub kind: ItemKind,
    pub body: PhysicsBody,
    sprite: Sprite,
}

impl Item {
    pub fn new(position: Vec2, params: ItemParams) -> Self {
        let mut world = storage::get_mut::<GameWorld>();

        let body = PhysicsBody::new(
            &mut world.collision_world,
            position,
            0.0,
            params.collider_size.as_f32(),
            true,
            true,
            None,
        );

        let sprite = Sprite::new(params.sprite);

        Item {
            id: params.id,
            kind: params.kind,
            body,
            sprite,
        }
    }

    fn physics_capabilities() -> PhysicsObject {
        fn active(_: HandleUntyped) -> bool {
            true
        }

        fn collider(handle: HandleUntyped) -> Rect {
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Item>();
            node.body.get_collider_rect()
        }

        fn set_speed_x(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Item>();

            node.body.velocity.x = speed;
        }

        fn set_speed_y(handle: HandleUntyped, speed: f32) {
            let mut node = scene::get_untyped_node(handle).unwrap().to_typed::<Item>();

            node.body.velocity.y = speed;
        }

        PhysicsObject {
            active,
            collider,
            set_speed_x,
            set_speed_y,
        }
    }

    fn network_update(mut node: RefMut<Self>) {
        node.body.update();
    }

    fn network_capabilities() -> NetworkReplicate {
        fn network_update(handle: HandleUntyped) {
            let node = scene::get_untyped_node(handle).unwrap().to_typed::<Item>();
            Item::network_update(node);
        }

        NetworkReplicate { network_update }
    }
}

impl Node for Item {
    fn ready(mut node: RefMut<Self>) {
        node.provides(Self::network_capabilities());
        node.provides(Self::physics_capabilities());
    }

    fn draw(node: RefMut<Self>) {
        node.sprite
            .draw(node.body.position, node.body.rotation, false, false);

        #[cfg(debug_assertions)]
        node.sprite.debug_draw(node.body.position);

        #[cfg(debug_assertions)]
        node.body.debug_draw();
    }
}
