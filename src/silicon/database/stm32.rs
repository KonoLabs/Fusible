//! STM32 Alternate Function (AF0-AF15) silicon pin tables.

use crate::silicon::model::{
    MuxFunction, PeripheralSignal, PinCapabilities, PinSpec, SiliconTarget,
};

macro_rules! mux {
    ($sig:expr, $af:expr, $name:expr) => {
        MuxFunction {
            signal: $sig,
            af_number: $af,
            af_name: $name,
        }
    };
}

const STM32_GPIO_CAP: PinCapabilities = PinCapabilities {
    is_gpio: true,
    is_analog: false,
    is_input_only: false,
    is_reserved_flash: false,
};

const STM32_ANALOG_CAP: PinCapabilities = PinCapabilities {
    is_gpio: true,
    is_analog: true,
    is_input_only: false,
    is_reserved_flash: false,
};

static PA0_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartTx(2), 7, "AF7")];
static PA1_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRx(2), 7, "AF7")];
static PA2_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartTx(2), 7, "AF7")];
static PA3_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRx(2), 7, "AF7")];
static PA5_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiSck(1), 5, "AF5")];
static PA6_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiMiso(1), 5, "AF5")];
static PA7_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiMosi(1), 5, "AF5")];
static PA9_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartTx(1), 7, "AF7"),
    mux!(PeripheralSignal::I2cScl(2), 4, "AF4"),
];
static PA10_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartRx(1), 7, "AF7"),
    mux!(PeripheralSignal::I2cSda(2), 4, "AF4"),
];
static PA11_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UsbDm, 10, "AF10"),
    mux!(PeripheralSignal::UartCts(1), 7, "AF7"),
];
static PA12_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UsbDp, 10, "AF10"),
    mux!(PeripheralSignal::UartRts(1), 7, "AF7"),
];
static PA13_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SwdDio, 0, "AF0")];
static PA14_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SwdClk, 0, "AF0")];

static PB3_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiSck(1), 5, "AF5")];
static PB4_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiMiso(1), 5, "AF5")];
static PB5_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::SpiMosi(1), 5, "AF5")];
static PB6_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cScl(1), 4, "AF4"),
    mux!(PeripheralSignal::UartTx(1), 7, "AF7"),
];
static PB7_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cSda(1), 4, "AF4"),
    mux!(PeripheralSignal::UartRx(1), 7, "AF7"),
];
static PB8_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::I2cScl(1), 4, "AF4")];
static PB9_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::I2cSda(1), 4, "AF4")];
static PB10_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cScl(2), 4, "AF4"),
    mux!(PeripheralSignal::UartTx(3), 7, "AF7"),
];
static PB11_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::I2cSda(2), 4, "AF4"),
    mux!(PeripheralSignal::UartRx(3), 7, "AF7"),
];

static PC4_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartTx(1), 7, "AF7")];
static PC5_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRx(1), 7, "AF7")];
static PC10_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartTx(4), 5, "AF5"),
    mux!(PeripheralSignal::UartTx(3), 7, "AF7"),
];
static PC11_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartRx(4), 5, "AF5"),
    mux!(PeripheralSignal::UartRx(3), 7, "AF7"),
];

pub static STM32G431_PINS: &[PinSpec] = &[
    PinSpec {
        gpio: None,
        name: "PA0",
        capabilities: STM32_ANALOG_CAP,
        strapping: None,
        functions: PA0_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA1",
        capabilities: STM32_ANALOG_CAP,
        strapping: None,
        functions: PA1_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA2",
        capabilities: STM32_ANALOG_CAP,
        strapping: None,
        functions: PA2_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA3",
        capabilities: STM32_ANALOG_CAP,
        strapping: None,
        functions: PA3_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA5",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PA5_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA6",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PA6_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA7",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PA7_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA9",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PA9_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA10",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PA10_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA11",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PA11_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA12",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PA12_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA13",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PA13_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PA14",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PA14_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PB3",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PB3_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PB4",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PB4_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PB5",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PB5_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PB6",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PB6_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PB7",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PB7_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PB8",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PB8_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PB9",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PB9_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PB10",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PB10_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PB11",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PB11_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PC4",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PC4_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PC5",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PC5_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PC10",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PC10_FUNCS,
    },
    PinSpec {
        gpio: None,
        name: "PC11",
        capabilities: STM32_GPIO_CAP,
        strapping: None,
        functions: PC11_FUNCS,
    },
];

pub static STM32G431: SiliconTarget = SiliconTarget {
    name: "stm32g431",
    family: "STM32",
    pins: STM32G431_PINS,
};
