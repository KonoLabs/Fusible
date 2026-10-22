//! Raspberry Pi RP2040 Function Select (F1-F9) and ADC silicon tables.

use crate::silicon::model::{
    MuxFunction, PeripheralSignal, PinCapabilities, PinSpec, SiliconTarget,
};

const RP_GPIO_CAP: PinCapabilities = PinCapabilities {
    is_gpio: true,
    is_analog: false,
    is_input_only: false,
    is_reserved_flash: false,
};

const RP_ADC_CAP: PinCapabilities = PinCapabilities {
    is_gpio: true,
    is_analog: true,
    is_input_only: false,
    is_reserved_flash: false,
};

static GP0_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(0), 1, "F1"),
    mux!(PeripheralSignal::UartTx(0), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(0, 'A'), 4, "F4"),
];

static GP1_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiCs(0, 0), 1, "F1"),
    mux!(PeripheralSignal::UartRx(0), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(0, 'B'), 4, "F4"),
];

static GP2_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiSck(0), 1, "F1"),
    mux!(PeripheralSignal::UartCts(0), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(1, 'A'), 4, "F4"),
];

static GP3_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMosi(0), 1, "F1"),
    mux!(PeripheralSignal::UartRts(0), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(1, 'B'), 4, "F4"),
];

static GP4_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(0), 1, "F1"),
    mux!(PeripheralSignal::UartTx(1), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(2, 'A'), 4, "F4"),
];

static GP5_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiCs(0, 0), 1, "F1"),
    mux!(PeripheralSignal::UartRx(1), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(2, 'B'), 4, "F4"),
];

static GP6_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiSck(0), 1, "F1"),
    mux!(PeripheralSignal::UartCts(1), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(3, 'A'), 4, "F4"),
];

static GP7_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMosi(0), 1, "F1"),
    mux!(PeripheralSignal::UartRts(1), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(3, 'B'), 4, "F4"),
];

static GP8_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(1), 1, "F1"),
    mux!(PeripheralSignal::UartTx(1), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(4, 'A'), 4, "F4"),
];

static GP9_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiCs(1, 0), 1, "F1"),
    mux!(PeripheralSignal::UartRx(1), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(4, 'B'), 4, "F4"),
];

static GP10_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiSck(1), 1, "F1"),
    mux!(PeripheralSignal::UartCts(1), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(5, 'A'), 4, "F4"),
];

static GP11_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMosi(1), 1, "F1"),
    mux!(PeripheralSignal::UartRts(1), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(5, 'B'), 4, "F4"),
];

static GP12_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(1), 1, "F1"),
    mux!(PeripheralSignal::UartTx(0), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(6, 'A'), 4, "F4"),
];

static GP13_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiCs(1, 0), 1, "F1"),
    mux!(PeripheralSignal::UartRx(0), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(6, 'B'), 4, "F4"),
];

static GP14_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiSck(1), 1, "F1"),
    mux!(PeripheralSignal::UartCts(0), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(7, 'A'), 4, "F4"),
];

static GP15_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMosi(1), 1, "F1"),
    mux!(PeripheralSignal::UartRts(0), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(7, 'B'), 4, "F4"),
];

static GP16_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(0), 1, "F1"),
    mux!(PeripheralSignal::UartTx(0), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(0, 'A'), 4, "F4"),
];

static GP17_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiCs(0, 0), 1, "F1"),
    mux!(PeripheralSignal::UartRx(0), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(0, 'B'), 4, "F4"),
];

static GP18_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiSck(0), 1, "F1"),
    mux!(PeripheralSignal::UartCts(0), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(1, 'A'), 4, "F4"),
];

static GP19_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMosi(0), 1, "F1"),
    mux!(PeripheralSignal::UartRts(0), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(1, 'B'), 4, "F4"),
];

static GP20_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiMiso(0), 1, "F1"),
    mux!(PeripheralSignal::UartTx(1), 2, "F2"),
    mux!(PeripheralSignal::I2cSda(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(2, 'A'), 4, "F4"),
];

static GP21_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::SpiCs(0, 0), 1, "F1"),
    mux!(PeripheralSignal::UartRx(1), 2, "F2"),
    mux!(PeripheralSignal::I2cScl(0), 3, "F3"),
    mux!(PeripheralSignal::Pwm(2, 'B'), 4, "F4"),
];

static GP26_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::Adc(0), 0, "ADC"),
    mux!(PeripheralSignal::I2cSda(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(5, 'A'), 4, "F4"),
];

static GP27_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::Adc(1), 0, "ADC"),
    mux!(PeripheralSignal::I2cScl(1), 3, "F3"),
    mux!(PeripheralSignal::Pwm(5, 'B'), 4, "F4"),
];

static GP28_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::Adc(2), 0, "ADC"),
    mux!(PeripheralSignal::UartTx(0), 2, "F2"),
    mux!(PeripheralSignal::Pwm(6, 'A'), 4, "F4"),
];

static GP29_FUNCS: &[MuxFunction] = &[
    mux!(PeripheralSignal::Adc(3), 0, "ADC"),
    mux!(PeripheralSignal::UartRx(0), 2, "F2"),
    mux!(PeripheralSignal::Pwm(6, 'B'), 4, "F4"),
];

pub static RP2040_PINS: &[PinSpec] = &[
    gpio_pin!(0, "GPIO0", RP_GPIO_CAP, GP0_FUNCS),
    gpio_pin!(1, "GPIO1", RP_GPIO_CAP, GP1_FUNCS),
    gpio_pin!(2, "GPIO2", RP_GPIO_CAP, GP2_FUNCS),
    gpio_pin!(3, "GPIO3", RP_GPIO_CAP, GP3_FUNCS),
    gpio_pin!(4, "GPIO4", RP_GPIO_CAP, GP4_FUNCS),
    gpio_pin!(5, "GPIO5", RP_GPIO_CAP, GP5_FUNCS),
    gpio_pin!(6, "GPIO6", RP_GPIO_CAP, GP6_FUNCS),
    gpio_pin!(7, "GPIO7", RP_GPIO_CAP, GP7_FUNCS),
    gpio_pin!(8, "GPIO8", RP_GPIO_CAP, GP8_FUNCS),
    gpio_pin!(9, "GPIO9", RP_GPIO_CAP, GP9_FUNCS),
    gpio_pin!(10, "GPIO10", RP_GPIO_CAP, GP10_FUNCS),
    gpio_pin!(11, "GPIO11", RP_GPIO_CAP, GP11_FUNCS),
    gpio_pin!(12, "GPIO12", RP_GPIO_CAP, GP12_FUNCS),
    gpio_pin!(13, "GPIO13", RP_GPIO_CAP, GP13_FUNCS),
    gpio_pin!(14, "GPIO14", RP_GPIO_CAP, GP14_FUNCS),
    gpio_pin!(15, "GPIO15", RP_GPIO_CAP, GP15_FUNCS),
    gpio_pin!(16, "GPIO16", RP_GPIO_CAP, GP16_FUNCS),
    gpio_pin!(17, "GPIO17", RP_GPIO_CAP, GP17_FUNCS),
    gpio_pin!(18, "GPIO18", RP_GPIO_CAP, GP18_FUNCS),
    gpio_pin!(19, "GPIO19", RP_GPIO_CAP, GP19_FUNCS),
    gpio_pin!(20, "GPIO20", RP_GPIO_CAP, GP20_FUNCS),
    gpio_pin!(21, "GPIO21", RP_GPIO_CAP, GP21_FUNCS),
    gpio_pin!(26, "GPIO26", RP_ADC_CAP, GP26_FUNCS),
    gpio_pin!(27, "GPIO27", RP_ADC_CAP, GP27_FUNCS),
    gpio_pin!(28, "GPIO28", RP_ADC_CAP, GP28_FUNCS),
    gpio_pin!(29, "GPIO29", RP_ADC_CAP, GP29_FUNCS),
];

pub static RP2040: SiliconTarget = SiliconTarget {
    name: "rp2040",
    family: "Raspberry Pi",
    pins: RP2040_PINS,
};
