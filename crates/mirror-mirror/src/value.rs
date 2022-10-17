use serde::{Deserialize, Serialize};
use speedy::{Readable, Writable};
use std::{any::Any, fmt};

use crate::{Enum, EnumValue, Reflect, Struct, StructValue};

#[derive(Readable, Writable, Serialize, Deserialize, Clone)]
pub struct Value(pub(crate) ValueInner);

impl Reflect for Value {
    fn as_any(&self) -> &dyn Any {
        self.0.as_any()
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self.0.as_any_mut()
    }

    fn as_reflect(&self) -> &dyn Reflect {
        self.0.as_reflect()
    }

    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self.0.as_reflect_mut()
    }

    fn patch(&mut self, value: &dyn Reflect) {
        self.0.patch(value)
    }

    fn clone_reflect(&self) -> Box<dyn Reflect> {
        Box::new(self.clone())
    }

    fn to_value(&self) -> Value {
        self.clone()
    }

    fn as_struct(&self) -> Option<&dyn Struct> {
        self.0.as_struct()
    }

    fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
        self.0.as_struct_mut()
    }

    fn as_enum(&self) -> Option<&dyn Enum> {
        self.0.as_enum()
    }

    fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
        self.0.as_enum_mut()
    }

    fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "{:#?}", self)
        } else {
            write!(f, "{:?}", self)
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

macro_rules! value_inner {
    (
        $(#[$m:meta])*
        pub(crate) enum ValueInner {
            $($ident:ident,)*
        }
    ) => {
        $(#[$m])*
        pub(crate) enum ValueInner {
            $($ident($ident),)*
        }

        impl Reflect for ValueInner {
            fn as_any(&self) -> &dyn Any {
                match self {
                    $(
                        Self::$ident(inner) => inner,
                    )*
                }
            }

            fn as_any_mut(&mut self) -> &mut dyn Any {
                match self {
                    $(
                        Self::$ident(inner) => inner,
                    )*
                }
            }

            fn as_reflect(&self) -> &dyn Reflect {
                self
            }

            fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
                self
            }

            fn patch(&mut self, value: &dyn Reflect) {
                match self {
                    $(
                        Self::$ident(inner) => {
                            if let Some(value) = value.downcast_ref::<$ident>() {
                                *inner = value.clone();
                            }
                        },
                    )*
                }
            }

            fn to_value(&self) -> Value {
                Value(self.clone())
            }

            fn clone_reflect(&self) -> Box<dyn Reflect> {
                Box::new(self.clone())
            }

            fn as_struct(&self) -> Option<&dyn Struct> {
                if let ValueInner::StructValue(value) = self {
                    Some(value)
                } else {
                    None
                }
            }

            fn as_struct_mut(&mut self) -> Option<&mut dyn Struct> {
                if let ValueInner::StructValue(value) = self {
                    Some(value)
                } else {
                    None
                }
            }

            fn as_enum(&self) -> Option<&dyn Enum> {
                if let ValueInner::EnumValue(value) = self {
                    Some(value)
                } else {
                    None
                }
            }

            fn as_enum_mut(&mut self) -> Option<&mut dyn Enum> {
                if let ValueInner::EnumValue(value) = self {
                    Some(value)
                } else {
                    None
                }
            }

            fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                if f.alternate() {
                    write!(f, "{:#?}", self)
                } else {
                    write!(f, "{:?}", self)
                }
            }
        }

        $(
            impl From<$ident> for Value {
                fn from(value: $ident) -> Self {
                    Self(ValueInner::$ident(value))
                }
            }
        )*
    };
}

value_inner! {
    #[allow(non_camel_case_types)]
    #[derive(Readable, Writable, Serialize, Deserialize, Debug, Clone)]
    pub(crate) enum ValueInner {
        usize,
        u8,
        u16,
        u32,
        u64,
        u128,
        i8,
        i16,
        i32,
        i64,
        i128,
        bool,
        char,
        f32,
        f64,
        String,
        StructValue,
        EnumValue,
    }
}
