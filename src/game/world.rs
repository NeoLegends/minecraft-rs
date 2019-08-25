use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct BlockPosition {
    pub x: i32,
    pub y: u16,
    pub z: i32,
}

#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct EntityId(pub i32);

#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct Rotation {
    pub pitch: u8,
    pub yaw: u8,
}

#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct MobType(pub i32);

#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct RotationFlipped {
    pub yaw: u8,
    pub pitch: u8,
}

#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct Uuid(u128);

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Vec3x32 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Vec3x64 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Copy, Clone, Debug, Deserialize, Hash, Eq, PartialEq, Serialize)]
pub struct Velocity {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

impl From<i32> for EntityId {
    #[inline]
    fn from(val: i32) -> Self {
        EntityId(val)
    }
}

impl From<EntityId> for i32 {
    #[inline]
    fn from(val: EntityId) -> Self {
        val.0
    }
}

impl From<i32> for MobType {
    #[inline]
    fn from(val: i32) -> Self {
        MobType(val)
    }
}

impl From<MobType> for i32 {
    #[inline]
    fn from(val: MobType) -> Self {
        val.0
    }
}

impl From<Rotation> for RotationFlipped {
    #[inline]
    fn from(val: Rotation) -> Self {
        RotationFlipped {
            pitch: val.pitch,
            yaw: val.yaw,
        }
    }
}

impl From<RotationFlipped> for Rotation {
    #[inline]
    fn from(val: RotationFlipped) -> Self {
        Rotation {
            pitch: val.pitch,
            yaw: val.yaw,
        }
    }
}
