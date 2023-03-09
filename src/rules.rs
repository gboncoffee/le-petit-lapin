//! Rule system for Lapin.

/// A window property. Currently supports only the class. Title
/// support is planned.
#[derive(Debug, PartialEq)]
pub enum Property {
    Class(String),
    // this is because someday it'll have support to the title
}

#[derive(Debug, PartialEq)]
/// What to apply to the window.
pub enum Apply {
    Workspace(usize),
    Fullscreen,
    Float,
}

#[derive(Debug, PartialEq)]
/// A rule to apply to a window on spawn.
pub struct Rule {
    /// A window property. Currently supports only the class. Title
    /// support is planned.
    pub property: Property,
    /// What to apply to the window.
    pub apply: Apply,
}

impl Rule {
    /// Creates a new rule. Not recommended, use the macro `rule!` instead.
    pub fn new(property: Property, apply: Apply) -> Self {
        Rule { property, apply }
    }
}

/// Macro to easily create rules
/// ```
/// use le_petit_lapin::*;
/// use le_petit_lapin::rules::*;
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
