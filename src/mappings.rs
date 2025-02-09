use evdev::{AbsoluteAxisCode, KeyCode};
use serde::{Deserialize, Serialize};

macro_rules! create_mapping {
    ( $name:ident, $mapper:ident, [ $( $btn:ident $(,)? )+ ] $(, $unknown:ident )? ) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
        pub enum $name {
            $(
                $btn,
            )+

            $(
                $unknown(u16),
            )?

            Stub
        }

        #[allow(unreachable_code)]
        impl From<$mapper> for $name {
            fn from(mapping: $mapper) -> Self {
                match mapping {
                    $(
                        $mapper::$btn => Self::$btn,
                    )+

                    _ => {
                        println!("mapping ({mapping:?}) missing counterpart");

                        $(
                            return Self::$unknown(mapping.code());
                        )?

                        panic!()
                    }
                }
            }
        }

        impl TryInto<$mapper> for $name {
            type Error = anyhow::Error;

            fn try_into(self) -> Result<$mapper, Self::Error> {
                match self {
                    $(
                        Self::$btn => Ok($mapper::$btn),
                    )+

                    $(
                        Self::$unknown(i) => Ok($mapper::new(i)),
                    )?

                    Self::Stub => Err(anyhow::anyhow!("Stub can't be matched")),
                }
            }
        }
    };
}

create_mapping!(
    Button,
    KeyCode,
    [
        BTN_0,
        BTN_1,
        BTN_2,
        BTN_3,
        BTN_4,
        BTN_5,
        BTN_6,
        BTN_7,
        BTN_8,
        BTN_9,
        BTN_LEFT,
        BTN_RIGHT,
        BTN_MIDDLE,
        BTN_TOP,
        BTN_TOP2,
        BTN_SIDE,
        BTN_TRIGGER,
        BTN_THUMB,
        BTN_THUMB2,
        BTN_PINKIE,
        BTN_BASE,
        BTN_BASE2,
        BTN_BASE3,
        BTN_BASE4,
        BTN_BASE5,
        BTN_BASE6,
        BTN_DEAD,
    ],
    Unknown
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
        ABS_BRAKE,
        ABS_HAT0X,
        ABS_HAT0Y,
        ABS_HAT1X,
        ABS_HAT1Y,
        ABS_HAT2X,
        ABS_HAT2Y,
    ]
);
