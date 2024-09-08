use bevy::prelude::*;
use bevy_mod_scripting::prelude::*;

use bevy_mod_scripting::lua::prelude::tealr::mlu::mlua::{UserData, UserDataFields};

use crate::{EntityRep, UserDataComponent};

pub struct N9Camera(pub Entity);

impl EntityRep for N9Camera {
    fn entity(&self) -> Entity {
        self.0
    }
}

impl UserData for N9Camera {
    fn add_fields<'lua, F: UserDataFields<'lua, Self>>(fields: &mut F) {
        Transform::add_fields::<'lua, Self, _>(fields);
    }
}
