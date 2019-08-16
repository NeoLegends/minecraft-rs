#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct BlockPosition {
    pub x: i32,
    pub y: u16,
    pub z: i32,
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
pub struct EntityId(pub i32);

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
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
