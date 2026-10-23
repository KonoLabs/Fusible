//! STM32G431 Alternate Function (AF0-AF15) silicon pin tables.
//!
//! Source: STM32G431x6/x8/xB datasheet DS12589 Rev 6 (October 2021),
//! <https://www.st.com/resource/en/datasheet/stm32g431c6.pdf>. Table 13
//! "Alternate function" gives the AF numbers and Table 12 "pin definition"
//! gives the ADC channels (ADCnm_INx entries bind to both ADCn and ADCm).
//!
//! Each package gets its own target listing only the GPIOs Table 12 bonds out
//! (pin count letter per Table 101: K = 32, C = 48/49, R = 64, M = 80,
//! V = 100 pins). `stm32g431c` lists the LQFP48 set, which UFQFPN48 and
//! WLCSP49 also bond out, and a bare `stm32g431` lists the 32-pin set,
//! which every package bonds out.
//!
//! Only signals that `PeripheralSignal` can express are listed: USART/UART
//! TX, RX, CTS and RTS, SPI SCK, MISO, MOSI and NSS, I2C SCL and SDA, SWD and
//! ADC channels. Timers, LPUART, SAI, FDCAN, UCPD, COMP, I2S and SMBA are omitted.
//! The G431 has no USB device peripheral, so no USB signals are listed.

mod port_a;
mod port_b;
mod port_c;
mod port_d;
mod port_e;
mod port_f;
mod port_g;

use crate::silicon::model::{PinCapabilities, PinSpec, SiliconTarget};

use port_a::*;
use port_b::*;
use port_c::*;
use port_d::*;
use port_e::*;
use port_f::*;
use port_g::*;

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

const PA0: PinSpec = port_pin!("PA0", STM32_ANALOG_CAP, PA0_FUNCS);
const PA1: PinSpec = port_pin!("PA1", STM32_ANALOG_CAP, PA1_FUNCS);
const PA2: PinSpec = port_pin!("PA2", STM32_ANALOG_CAP, PA2_FUNCS);
const PA3: PinSpec = port_pin!("PA3", STM32_ANALOG_CAP, PA3_FUNCS);
const PA4: PinSpec = port_pin!("PA4", STM32_ANALOG_CAP, PA4_FUNCS);
const PA5: PinSpec = port_pin!("PA5", STM32_ANALOG_CAP, PA5_FUNCS);
const PA6: PinSpec = port_pin!("PA6", STM32_ANALOG_CAP, PA6_FUNCS);
const PA7: PinSpec = port_pin!("PA7", STM32_ANALOG_CAP, PA7_FUNCS);
const PA8: PinSpec = port_pin!("PA8", STM32_GPIO_CAP, PA8_FUNCS);
const PA9: PinSpec = port_pin!("PA9", STM32_GPIO_CAP, PA9_FUNCS);
const PA10: PinSpec = port_pin!("PA10", STM32_GPIO_CAP, PA10_FUNCS);
const PA11: PinSpec = port_pin!("PA11", STM32_GPIO_CAP, PA11_FUNCS);
const PA12: PinSpec = port_pin!("PA12", STM32_GPIO_CAP, PA12_FUNCS);
const PA13: PinSpec = port_pin!("PA13", STM32_GPIO_CAP, PA13_FUNCS);
const PA14: PinSpec = port_pin!("PA14", STM32_GPIO_CAP, PA14_FUNCS);
const PA15: PinSpec = port_pin!("PA15", STM32_GPIO_CAP, PA15_FUNCS);
const PB0: PinSpec = port_pin!("PB0", STM32_ANALOG_CAP, PB0_FUNCS);
const PB1: PinSpec = port_pin!("PB1", STM32_ANALOG_CAP, PB1_FUNCS);
const PB2: PinSpec = port_pin!("PB2", STM32_ANALOG_CAP, PB2_FUNCS);
const PB3: PinSpec = port_pin!("PB3", STM32_GPIO_CAP, PB3_FUNCS);
const PB4: PinSpec = port_pin!("PB4", STM32_GPIO_CAP, PB4_FUNCS);
const PB5: PinSpec = port_pin!("PB5", STM32_GPIO_CAP, PB5_FUNCS);
const PB6: PinSpec = port_pin!("PB6", STM32_GPIO_CAP, PB6_FUNCS);
const PB7: PinSpec = port_pin!("PB7", STM32_GPIO_CAP, PB7_FUNCS);
const PB8: PinSpec = port_pin!("PB8", STM32_GPIO_CAP, PB8_FUNCS);
const PB9: PinSpec = port_pin!("PB9", STM32_GPIO_CAP, PB9_FUNCS);
const PB10: PinSpec = port_pin!("PB10", STM32_GPIO_CAP, PB10_FUNCS);
const PB11: PinSpec = port_pin!("PB11", STM32_ANALOG_CAP, PB11_FUNCS);
const PB12: PinSpec = port_pin!("PB12", STM32_ANALOG_CAP, PB12_FUNCS);
const PB13: PinSpec = port_pin!("PB13", STM32_GPIO_CAP, PB13_FUNCS);
const PB14: PinSpec = port_pin!("PB14", STM32_ANALOG_CAP, PB14_FUNCS);
const PB15: PinSpec = port_pin!("PB15", STM32_ANALOG_CAP, PB15_FUNCS);
const PC0: PinSpec = port_pin!("PC0", STM32_ANALOG_CAP, PC0_FUNCS);
const PC1: PinSpec = port_pin!("PC1", STM32_ANALOG_CAP, PC1_FUNCS);
const PC2: PinSpec = port_pin!("PC2", STM32_ANALOG_CAP, PC2_FUNCS);
const PC3: PinSpec = port_pin!("PC3", STM32_ANALOG_CAP, PC3_FUNCS);
const PC4: PinSpec = port_pin!("PC4", STM32_ANALOG_CAP, PC4_FUNCS);
const PC5: PinSpec = port_pin!("PC5", STM32_ANALOG_CAP, PC5_FUNCS);
const PC6: PinSpec = port_pin!("PC6", STM32_GPIO_CAP, PC6_FUNCS);
const PC7: PinSpec = port_pin!("PC7", STM32_GPIO_CAP, PC7_FUNCS);
const PC8: PinSpec = port_pin!("PC8", STM32_GPIO_CAP, PC8_FUNCS);
const PC9: PinSpec = port_pin!("PC9", STM32_GPIO_CAP, PC9_FUNCS);
const PC10: PinSpec = port_pin!("PC10", STM32_GPIO_CAP, PC10_FUNCS);
const PC11: PinSpec = port_pin!("PC11", STM32_GPIO_CAP, PC11_FUNCS);
const PC12: PinSpec = port_pin!("PC12", STM32_GPIO_CAP, PC12_FUNCS);
const PC13: PinSpec = port_pin!("PC13", STM32_GPIO_CAP, PC13_FUNCS);
const PC14: PinSpec = port_pin!("PC14", STM32_GPIO_CAP, PC14_FUNCS);
const PC15: PinSpec = port_pin!("PC15", STM32_GPIO_CAP, PC15_FUNCS);
const PD0: PinSpec = port_pin!("PD0", STM32_GPIO_CAP, PD0_FUNCS);
const PD1: PinSpec = port_pin!("PD1", STM32_GPIO_CAP, PD1_FUNCS);
const PD2: PinSpec = port_pin!("PD2", STM32_GPIO_CAP, PD2_FUNCS);
const PD3: PinSpec = port_pin!("PD3", STM32_GPIO_CAP, PD3_FUNCS);
const PD4: PinSpec = port_pin!("PD4", STM32_GPIO_CAP, PD4_FUNCS);
const PD5: PinSpec = port_pin!("PD5", STM32_GPIO_CAP, PD5_FUNCS);
const PD6: PinSpec = port_pin!("PD6", STM32_GPIO_CAP, PD6_FUNCS);
const PD7: PinSpec = port_pin!("PD7", STM32_GPIO_CAP, PD7_FUNCS);
const PD8: PinSpec = port_pin!("PD8", STM32_GPIO_CAP, PD8_FUNCS);
const PD9: PinSpec = port_pin!("PD9", STM32_GPIO_CAP, PD9_FUNCS);
const PD10: PinSpec = port_pin!("PD10", STM32_GPIO_CAP, PD10_FUNCS);
const PD11: PinSpec = port_pin!("PD11", STM32_GPIO_CAP, PD11_FUNCS);
const PD12: PinSpec = port_pin!("PD12", STM32_GPIO_CAP, PD12_FUNCS);
const PD13: PinSpec = port_pin!("PD13", STM32_GPIO_CAP, PD13_FUNCS);
const PD14: PinSpec = port_pin!("PD14", STM32_GPIO_CAP, PD14_FUNCS);
const PD15: PinSpec = port_pin!("PD15", STM32_GPIO_CAP, PD15_FUNCS);
const PE0: PinSpec = port_pin!("PE0", STM32_GPIO_CAP, PE0_FUNCS);
const PE1: PinSpec = port_pin!("PE1", STM32_GPIO_CAP, PE1_FUNCS);
const PE2: PinSpec = port_pin!("PE2", STM32_GPIO_CAP, PE2_FUNCS);
const PE3: PinSpec = port_pin!("PE3", STM32_GPIO_CAP, PE3_FUNCS);
const PE4: PinSpec = port_pin!("PE4", STM32_GPIO_CAP, PE4_FUNCS);
const PE5: PinSpec = port_pin!("PE5", STM32_GPIO_CAP, PE5_FUNCS);
const PE6: PinSpec = port_pin!("PE6", STM32_GPIO_CAP, PE6_FUNCS);
const PE7: PinSpec = port_pin!("PE7", STM32_GPIO_CAP, PE7_FUNCS);
const PE8: PinSpec = port_pin!("PE8", STM32_GPIO_CAP, PE8_FUNCS);
const PE9: PinSpec = port_pin!("PE9", STM32_GPIO_CAP, PE9_FUNCS);
const PE10: PinSpec = port_pin!("PE10", STM32_GPIO_CAP, PE10_FUNCS);
const PE11: PinSpec = port_pin!("PE11", STM32_GPIO_CAP, PE11_FUNCS);
const PE12: PinSpec = port_pin!("PE12", STM32_GPIO_CAP, PE12_FUNCS);
const PE13: PinSpec = port_pin!("PE13", STM32_GPIO_CAP, PE13_FUNCS);
const PE14: PinSpec = port_pin!("PE14", STM32_GPIO_CAP, PE14_FUNCS);
const PE15: PinSpec = port_pin!("PE15", STM32_GPIO_CAP, PE15_FUNCS);
const PF0: PinSpec = port_pin!("PF0", STM32_ANALOG_CAP, PF0_FUNCS);
const PF1: PinSpec = port_pin!("PF1", STM32_ANALOG_CAP, PF1_FUNCS);
const PF2: PinSpec = port_pin!("PF2", STM32_GPIO_CAP, PF2_FUNCS);
const PF9: PinSpec = port_pin!("PF9", STM32_GPIO_CAP, PF9_FUNCS);
const PF10: PinSpec = port_pin!("PF10", STM32_GPIO_CAP, PF10_FUNCS);
const PG10: PinSpec = port_pin!("PG10", STM32_GPIO_CAP, PG10_FUNCS);

/// LQFP32 and UFQFPN32 (`stm32g431k`): 26 GPIOs.
static PINS_32: &[PinSpec] = &[
    PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10, PA11, PA12, PA13, PA14, PA15, PB0, PB3,
    PB4, PB5, PB6, PB7, PB8, PF0, PF1, PG10,
];

/// LQFP48 (`stm32g431c`), also bonded on UFQFPN48 and WLCSP49: 38 GPIOs.
static PINS_48: &[PinSpec] = &[
    PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10, PA11, PA12, PA13, PA14, PA15, PB0, PB1,
    PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15, PC13, PC14, PC15,
    PF0, PF1, PG10,
];

/// LQFP64 and UFBGA64 (`stm32g431r`): 52 GPIOs.
static PINS_64: &[PinSpec] = &[
    PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10, PA11, PA12, PA13, PA14, PA15, PB0, PB1,
    PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15, PC0, PC1, PC2, PC3,
    PC4, PC5, PC6, PC7, PC8, PC9, PC10, PC11, PC12, PC13, PC14, PC15, PD2, PF0, PF1, PG10,
];

/// LQFP80 (`stm32g431m`): 66 GPIOs.
static PINS_80: &[PinSpec] = &[
    PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10, PA11, PA12, PA13, PA14, PA15, PB0, PB1,
    PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15, PC0, PC1, PC2, PC3,
    PC4, PC5, PC6, PC7, PC8, PC9, PC10, PC11, PC12, PC13, PC14, PC15, PD0, PD1, PD2, PD8, PD9,
    PD10, PE7, PE8, PE9, PE10, PE11, PE12, PE13, PE14, PE15, PF0, PF1, PG10,
];

/// LQFP100 (`stm32g431v`): 86 GPIOs.
static PINS_100: &[PinSpec] = &[
    PA0, PA1, PA2, PA3, PA4, PA5, PA6, PA7, PA8, PA9, PA10, PA11, PA12, PA13, PA14, PA15, PB0, PB1,
    PB2, PB3, PB4, PB5, PB6, PB7, PB8, PB9, PB10, PB11, PB12, PB13, PB14, PB15, PC0, PC1, PC2, PC3,
    PC4, PC5, PC6, PC7, PC8, PC9, PC10, PC11, PC12, PC13, PC14, PC15, PD0, PD1, PD2, PD3, PD4, PD5,
    PD6, PD7, PD8, PD9, PD10, PD11, PD12, PD13, PD14, PD15, PE0, PE1, PE2, PE3, PE4, PE5, PE6, PE7,
    PE8, PE9, PE10, PE11, PE12, PE13, PE14, PE15, PF0, PF1, PF2, PF9, PF10, PG10,
];

const fn target(name: &'static str, pins: &'static [PinSpec]) -> SiliconTarget {
    SiliconTarget {
        name,
        family: "STM32",
        pins,
    }
}

/// STM32G431 with no package suffix: only the GPIOs every package bonds out.
pub static STM32G431: SiliconTarget = target("stm32g431", PINS_32);
pub static STM32G431K: SiliconTarget = target("stm32g431k", PINS_32);
pub static STM32G431C: SiliconTarget = target("stm32g431c", PINS_48);
pub static STM32G431R: SiliconTarget = target("stm32g431r", PINS_64);
pub static STM32G431M: SiliconTarget = target("stm32g431m", PINS_80);
pub static STM32G431V: SiliconTarget = target("stm32g431v", PINS_100);
