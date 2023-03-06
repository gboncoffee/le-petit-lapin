//! Rule system for Lapin.

#[derive(Debug, PartialEq)]
pub enum Property {
    Class(String),
    // this is because someday it'll have support to the title
}

#[derive(Debug, PartialEq)]
pub enum Apply {
    Workspace(usize),
    Fullscreen,
    Float,
}

#[derive(Debug, PartialEq)]
pub struct Rule {
    pub property: Property,
    pub apply: Apply,
}

impl Rule {
    pub fn new(property: Property, apply: Apply) -> Self {
        Rule { property, apply }
    }
}

/// Macro to easily create rules
/// ```
/// use lapin::*;
/// use lapin::rules::*;
/// rule!(class "Gimp" => Apply::Fullscreen);
/// rule!(class "QjackCtl" => Apply::Float);
/// ```
#[macro_export]
macro_rules! rule {
    (class $name:literal => $apply:expr) => {
        Rule {
            property: Property::Class(String::from($name)),
            apply: $apply,
        }
    }; // (title $name:literal => $apply:expr) => {
       //     Rule { property: Property::Title(String::from($name)), apply: $apply }
       // };
}
