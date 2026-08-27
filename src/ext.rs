#[cfg(feature = "scripting")]
use bevy_mod_scripting::bindings::ScriptValue;

pub trait ValueExt {
    fn to_f32(&self) -> Option<f32>;

    fn to_f32_or<E>(&self, err: E) -> Result<f32, E> {
        self.to_f32().ok_or(err)
    }
}

#[cfg(feature = "scripting")]
impl ValueExt for ScriptValue {
    fn to_f32(&self) -> Option<f32> {
        match self {
            ScriptValue::Float(f) => Some(*f as f32),
            ScriptValue::Integer(i) => Some(*i as f32),
            _ => None,
        }
    }
}
