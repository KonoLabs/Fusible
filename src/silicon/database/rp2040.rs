//! Raspberry Pi RP2040 Function Select (F1-F9) and ADC silicon tables.

use crate::silicon::model::{
    GpioNum, MuxFunction, PeripheralSignal, PinCapabilities, PinSpec, SiliconTarget,
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
    PinSpec {
        gpio: Some(GpioNum(0)),
        name: "GPIO0",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP0_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(1)),
        name: "GPIO1",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP1_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(2)),
        name: "GPIO2",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP2_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(3)),
        name: "GPIO3",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP3_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(4)),
        name: "GPIO4",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP4_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(5)),
        name: "GPIO5",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP5_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(6)),
        name: "GPIO6",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP6_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(7)),
        name: "GPIO7",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP7_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(8)),
        name: "GPIO8",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP8_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(9)),
        name: "GPIO9",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP9_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(10)),
        name: "GPIO10",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP10_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(11)),
        name: "GPIO11",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP11_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(12)),
        name: "GPIO12",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP12_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(13)),
        name: "GPIO13",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP13_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(14)),
        name: "GPIO14",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP14_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(15)),
        name: "GPIO15",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP15_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(16)),
        name: "GPIO16",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP16_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(17)),
        name: "GPIO17",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP17_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(18)),
        name: "GPIO18",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP18_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(19)),
        name: "GPIO19",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP19_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(20)),
        name: "GPIO20",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP20_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(21)),
        name: "GPIO21",
        capabilities: RP_GPIO_CAP,
        strapping: None,
        functions: GP21_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(26)),
        name: "GPIO26",
        capabilities: RP_ADC_CAP,
        strapping: None,
        functions: GP26_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(27)),
        name: "GPIO27",
        capabilities: RP_ADC_CAP,
        strapping: None,
        functions: GP27_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(28)),
        name: "GPIO28",
        capabilities: RP_ADC_CAP,
        strapping: None,
        functions: GP28_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(29)),
        name: "GPIO29",
        capabilities: RP_ADC_CAP,
        strapping: None,
        functions: GP29_FUNCS,
    },
];

pub static RP2040: SiliconTarget = SiliconTarget {
    name: "rp2040",
    family: "Raspberry Pi",
    pins: RP2040_PINS,
};
