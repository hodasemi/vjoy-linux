use evdev::{AbsoluteAxisCode, KeyCode};
use serde::{Deserialize, Serialize};

macro_rules! create_mapping {
    ( $name:ident, $mapper:ident, [ $( $btn:ident $(,)? )+ ] ) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
        pub enum $name {
            $(
                $btn,
            )+

            Stub
        }

        impl From<$mapper> for $name {
            fn from(key_code: $mapper) -> Self {
                match key_code {
                    $(
                        $mapper::$btn => Self::$btn,
                    )+
                    _ => panic!()
                }
            }
        }

        impl Into<$mapper> for $name {
            fn into(self) -> $mapper {
                match self {
                    $(
                        Self::$btn => $mapper::$btn,
                    )+

                    Self::Stub => panic!("Stub can't be matched"),
                }
            }
        }
    };
}

create_mapping!(
    Button,
    KeyCode,
    [
        BTN_0, BTN_1, BTN_2, BTN_3, BTN_4, BTN_5, BTN_6, BTN_7, BTN_8, BTN_9, BTN_LEFT, BTN_RIGHT,
        BTN_MIDDLE, BTN_SIDE
    ]
);

create_mapping!(
    Axis,
    AbsoluteAxisCode,
    [
        ABS_X,
        ABS_Y,
        ABS_Z,
        ABS_RX,
        ABS_RY,
        ABS_RZ,
        ABS_THROTTLE,
        ABS_RUDDER,
        ABS_WHEEL,
        ABS_BRAKE
    ]
);
