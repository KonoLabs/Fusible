//! Espressif ESP32-S3 IO MUX, boot strapping, and reserved memory pin tables.

use crate::silicon::model::{
    GpioNum, MuxFunction, PeripheralSignal, PinCapabilities, PinSpec, SiliconTarget,
    StrappingRequirement,
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

const ESP_GPIO_CAP: PinCapabilities = PinCapabilities {
    is_gpio: true,
    is_analog: false,
    is_input_only: false,
    is_reserved_flash: false,
};

const ESP_INPUT_ONLY: PinCapabilities = PinCapabilities {
    is_gpio: true,
    is_analog: false,
    is_input_only: true,
    is_reserved_flash: false,
};

const ESP_FLASH_CAP: PinCapabilities = PinCapabilities {
    is_gpio: false,
    is_analog: false,
    is_input_only: false,
    is_reserved_flash: true,
};

static GP0_FUNCS: &[MuxFunction] = &[];
static GP3_FUNCS: &[MuxFunction] = &[];
static GP19_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UsbDm, 0, "USB_DM")];
static GP20_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UsbDp, 0, "USB_DP")];
static GP43_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartTx(0), 0, "U0TXD")];
static GP44_FUNCS: &[MuxFunction] = &[mux!(PeripheralSignal::UartRx(0), 0, "U0RXD")];
static GP45_FUNCS: &[MuxFunction] = &[];
static GP46_FUNCS: &[MuxFunction] = &[];
static EMPTY_FUNCS: &[MuxFunction] = &[];

pub static ESP32S3_PINS: &[PinSpec] = &[
    PinSpec {
        gpio: Some(GpioNum(0)),
        name: "GPIO0",
        capabilities: ESP_GPIO_CAP,
        strapping: Some(StrappingRequirement::MustBeHigh {
            purpose: "SPI Flash Boot mode",
            violation_consequence: "holding GPIO0 LOW at reset forces ROM download bootloader",
        }),
        functions: GP0_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(1)),
        name: "GPIO1",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(2)),
        name: "GPIO2",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(3)),
        name: "GPIO3",
        capabilities: ESP_GPIO_CAP,
        strapping: Some(StrappingRequirement::MustBeLow {
            purpose: "JTAG signal selection",
            violation_consequence: "holding GPIO3 HIGH at reset alters JTAG routing",
        }),
        functions: GP3_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(4)),
        name: "GPIO4",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(5)),
        name: "GPIO5",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(6)),
        name: "GPIO6",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(7)),
        name: "GPIO7",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(8)),
        name: "GPIO8",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(9)),
        name: "GPIO9",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(10)),
        name: "GPIO10",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(11)),
        name: "GPIO11",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(12)),
        name: "GPIO12",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(13)),
        name: "GPIO13",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(14)),
        name: "GPIO14",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(15)),
        name: "GPIO15",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(16)),
        name: "GPIO16",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(17)),
        name: "GPIO17",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(18)),
        name: "GPIO18",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(19)),
        name: "GPIO19",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: GP19_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(20)),
        name: "GPIO20",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: GP20_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(21)),
        name: "GPIO21",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    // GPIO 26-32: Reserved for SPI Flash
    PinSpec {
        gpio: Some(GpioNum(26)),
        name: "GPIO26",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(27)),
        name: "GPIO27",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(28)),
        name: "GPIO28",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(29)),
        name: "GPIO29",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(30)),
        name: "GPIO30",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(31)),
        name: "GPIO31",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(32)),
        name: "GPIO32",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    // GPIO 33-37: Reserved for Octal Flash / PSRAM
    PinSpec {
        gpio: Some(GpioNum(33)),
        name: "GPIO33",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(34)),
        name: "GPIO34",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(35)),
        name: "GPIO35",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(36)),
        name: "GPIO36",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(37)),
        name: "GPIO37",
        capabilities: ESP_FLASH_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(38)),
        name: "GPIO38",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(39)),
        name: "GPIO39",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(40)),
        name: "GPIO40",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(41)),
        name: "GPIO41",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(42)),
        name: "GPIO42",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(43)),
        name: "GPIO43",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: GP43_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(44)),
        name: "GPIO44",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: GP44_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(45)),
        name: "GPIO45",
        capabilities: ESP_GPIO_CAP,
        strapping: Some(StrappingRequirement::MustBeLow {
            purpose: "VDD_SPI Flash voltage selection",
            violation_consequence:
                "holding GPIO45 HIGH selects 1.8V flash and browns out standard 3.3V modules",
        }),
        functions: GP45_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(46)),
        name: "GPIO46",
        capabilities: ESP_INPUT_ONLY,
        strapping: Some(StrappingRequirement::MustBeLow {
            purpose: "ROM boot log output",
            violation_consequence:
                "holding GPIO46 HIGH suppresses ROM boot messages; pin cannot be used as an output",
        }),
        functions: GP46_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(47)),
        name: "GPIO47",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
    PinSpec {
        gpio: Some(GpioNum(48)),
        name: "GPIO48",
        capabilities: ESP_GPIO_CAP,
        strapping: None,
        functions: EMPTY_FUNCS,
    },
];

pub static ESP32S3: SiliconTarget = SiliconTarget {
    name: "esp32-s3",
    family: "Espressif",
    pins: ESP32S3_PINS,
};
