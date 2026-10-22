//! STM32 Alternate Function (AF0-AF15) silicon pin tables.

use crate::silicon::model::{
    MuxFunction, PeripheralSignal, PinCapabilities, PinSpec, SiliconTarget,
};

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

static PA0_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartTx(2), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 1), 0, "ADC"),
    mux!(PeripheralSignal::AdcChannel(2, 1), 0, "ADC"),
];
static PA1_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartRx(2), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 2), 0, "ADC"),
    mux!(PeripheralSignal::AdcChannel(2, 2), 0, "ADC"),
];
static PA2_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartTx(2), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 3), 0, "ADC"),
];
static PA3_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::UartRx(2), 7, "AF7"),
    mux!(PeripheralSignal::AdcChannel(1, 4), 0, "ADC"),
];
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
    port_pin!("PA0", STM32_ANALOG_CAP, PA0_FUNCS),
    port_pin!("PA1", STM32_ANALOG_CAP, PA1_FUNCS),
    port_pin!("PA2", STM32_ANALOG_CAP, PA2_FUNCS),
    port_pin!("PA3", STM32_ANALOG_CAP, PA3_FUNCS),
    port_pin!("PA5", STM32_GPIO_CAP, PA5_FUNCS),
    port_pin!("PA6", STM32_GPIO_CAP, PA6_FUNCS),
    port_pin!("PA7", STM32_GPIO_CAP, PA7_FUNCS),
    port_pin!("PA9", STM32_GPIO_CAP, PA9_FUNCS),
    port_pin!("PA10", STM32_GPIO_CAP, PA10_FUNCS),
    port_pin!("PA11", STM32_GPIO_CAP, PA11_FUNCS),
    port_pin!("PA12", STM32_GPIO_CAP, PA12_FUNCS),
    port_pin!("PA13", STM32_GPIO_CAP, PA13_FUNCS),
    port_pin!("PA14", STM32_GPIO_CAP, PA14_FUNCS),
    port_pin!("PB3", STM32_GPIO_CAP, PB3_FUNCS),
    port_pin!("PB4", STM32_GPIO_CAP, PB4_FUNCS),
    port_pin!("PB5", STM32_GPIO_CAP, PB5_FUNCS),
    port_pin!("PB6", STM32_GPIO_CAP, PB6_FUNCS),
    port_pin!("PB7", STM32_GPIO_CAP, PB7_FUNCS),
    port_pin!("PB8", STM32_GPIO_CAP, PB8_FUNCS),
    port_pin!("PB9", STM32_GPIO_CAP, PB9_FUNCS),
    port_pin!("PB10", STM32_GPIO_CAP, PB10_FUNCS),
    port_pin!("PB11", STM32_GPIO_CAP, PB11_FUNCS),
    port_pin!("PC4", STM32_GPIO_CAP, PC4_FUNCS),
    port_pin!("PC5", STM32_GPIO_CAP, PC5_FUNCS),
    port_pin!("PC10", STM32_GPIO_CAP, PC10_FUNCS),
    port_pin!("PC11", STM32_GPIO_CAP, PC11_FUNCS),
];

pub static STM32G431: SiliconTarget = SiliconTarget {
    name: "stm32g431",
    family: "STM32",
    pins: STM32G431_PINS,
};
