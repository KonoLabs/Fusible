//! STM32G431 Alternate Function (AF0-AF15) silicon pin tables.
//!
//! Source: STM32G431x6/x8/xB datasheet DS12589 Rev 6 (October 2021),
//! <https://www.st.com/resource/en/datasheet/stm32g431c6.pdf>. Table 13
//! "Alternate function" gives the AF numbers and Table 12 "pin definition"
//! gives the ADC channels (ADCnm_INx entries bind to both ADCn and ADCm).
//!
//! The table covers all 86 GPIOs of the largest package (LQFP100, `stm32g431v`).
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

pub static STM32G431_PINS: &[PinSpec] = &[
    port_pin!("PA0", STM32_ANALOG_CAP, PA0_FUNCS),
    port_pin!("PA1", STM32_ANALOG_CAP, PA1_FUNCS),
    port_pin!("PA2", STM32_ANALOG_CAP, PA2_FUNCS),
    port_pin!("PA3", STM32_ANALOG_CAP, PA3_FUNCS),
    port_pin!("PA4", STM32_ANALOG_CAP, PA4_FUNCS),
    port_pin!("PA5", STM32_ANALOG_CAP, PA5_FUNCS),
    port_pin!("PA6", STM32_ANALOG_CAP, PA6_FUNCS),
    port_pin!("PA7", STM32_ANALOG_CAP, PA7_FUNCS),
    port_pin!("PA8", STM32_GPIO_CAP, PA8_FUNCS),
    port_pin!("PA9", STM32_GPIO_CAP, PA9_FUNCS),
    port_pin!("PA10", STM32_GPIO_CAP, PA10_FUNCS),
    port_pin!("PA11", STM32_GPIO_CAP, PA11_FUNCS),
    port_pin!("PA12", STM32_GPIO_CAP, PA12_FUNCS),
    port_pin!("PA13", STM32_GPIO_CAP, PA13_FUNCS),
    port_pin!("PA14", STM32_GPIO_CAP, PA14_FUNCS),
    port_pin!("PA15", STM32_GPIO_CAP, PA15_FUNCS),
    port_pin!("PB0", STM32_ANALOG_CAP, PB0_FUNCS),
    port_pin!("PB1", STM32_ANALOG_CAP, PB1_FUNCS),
    port_pin!("PB2", STM32_ANALOG_CAP, PB2_FUNCS),
    port_pin!("PB3", STM32_GPIO_CAP, PB3_FUNCS),
    port_pin!("PB4", STM32_GPIO_CAP, PB4_FUNCS),
    port_pin!("PB5", STM32_GPIO_CAP, PB5_FUNCS),
    port_pin!("PB6", STM32_GPIO_CAP, PB6_FUNCS),
    port_pin!("PB7", STM32_GPIO_CAP, PB7_FUNCS),
    port_pin!("PB8", STM32_GPIO_CAP, PB8_FUNCS),
    port_pin!("PB9", STM32_GPIO_CAP, PB9_FUNCS),
    port_pin!("PB10", STM32_GPIO_CAP, PB10_FUNCS),
    port_pin!("PB11", STM32_ANALOG_CAP, PB11_FUNCS),
    port_pin!("PB12", STM32_ANALOG_CAP, PB12_FUNCS),
    port_pin!("PB13", STM32_GPIO_CAP, PB13_FUNCS),
    port_pin!("PB14", STM32_ANALOG_CAP, PB14_FUNCS),
    port_pin!("PB15", STM32_ANALOG_CAP, PB15_FUNCS),
    port_pin!("PC0", STM32_ANALOG_CAP, PC0_FUNCS),
    port_pin!("PC1", STM32_ANALOG_CAP, PC1_FUNCS),
    port_pin!("PC2", STM32_ANALOG_CAP, PC2_FUNCS),
    port_pin!("PC3", STM32_ANALOG_CAP, PC3_FUNCS),
    port_pin!("PC4", STM32_ANALOG_CAP, PC4_FUNCS),
    port_pin!("PC5", STM32_ANALOG_CAP, PC5_FUNCS),
    port_pin!("PC6", STM32_GPIO_CAP, PC6_FUNCS),
    port_pin!("PC7", STM32_GPIO_CAP, PC7_FUNCS),
    port_pin!("PC8", STM32_GPIO_CAP, PC8_FUNCS),
    port_pin!("PC9", STM32_GPIO_CAP, PC9_FUNCS),
    port_pin!("PC10", STM32_GPIO_CAP, PC10_FUNCS),
    port_pin!("PC11", STM32_GPIO_CAP, PC11_FUNCS),
    port_pin!("PC12", STM32_GPIO_CAP, PC12_FUNCS),
    port_pin!("PC13", STM32_GPIO_CAP, PC13_FUNCS),
    port_pin!("PC14", STM32_GPIO_CAP, PC14_FUNCS),
    port_pin!("PC15", STM32_GPIO_CAP, PC15_FUNCS),
    port_pin!("PD0", STM32_GPIO_CAP, PD0_FUNCS),
    port_pin!("PD1", STM32_GPIO_CAP, PD1_FUNCS),
    port_pin!("PD2", STM32_GPIO_CAP, PD2_FUNCS),
    port_pin!("PD3", STM32_GPIO_CAP, PD3_FUNCS),
    port_pin!("PD4", STM32_GPIO_CAP, PD4_FUNCS),
    port_pin!("PD5", STM32_GPIO_CAP, PD5_FUNCS),
    port_pin!("PD6", STM32_GPIO_CAP, PD6_FUNCS),
    port_pin!("PD7", STM32_GPIO_CAP, PD7_FUNCS),
    port_pin!("PD8", STM32_GPIO_CAP, PD8_FUNCS),
    port_pin!("PD9", STM32_GPIO_CAP, PD9_FUNCS),
    port_pin!("PD10", STM32_GPIO_CAP, PD10_FUNCS),
    port_pin!("PD11", STM32_GPIO_CAP, PD11_FUNCS),
    port_pin!("PD12", STM32_GPIO_CAP, PD12_FUNCS),
    port_pin!("PD13", STM32_GPIO_CAP, PD13_FUNCS),
    port_pin!("PD14", STM32_GPIO_CAP, PD14_FUNCS),
    port_pin!("PD15", STM32_GPIO_CAP, PD15_FUNCS),
    port_pin!("PE0", STM32_GPIO_CAP, PE0_FUNCS),
    port_pin!("PE1", STM32_GPIO_CAP, PE1_FUNCS),
    port_pin!("PE2", STM32_GPIO_CAP, PE2_FUNCS),
    port_pin!("PE3", STM32_GPIO_CAP, PE3_FUNCS),
    port_pin!("PE4", STM32_GPIO_CAP, PE4_FUNCS),
    port_pin!("PE5", STM32_GPIO_CAP, PE5_FUNCS),
    port_pin!("PE6", STM32_GPIO_CAP, PE6_FUNCS),
    port_pin!("PE7", STM32_GPIO_CAP, PE7_FUNCS),
    port_pin!("PE8", STM32_GPIO_CAP, PE8_FUNCS),
    port_pin!("PE9", STM32_GPIO_CAP, PE9_FUNCS),
    port_pin!("PE10", STM32_GPIO_CAP, PE10_FUNCS),
    port_pin!("PE11", STM32_GPIO_CAP, PE11_FUNCS),
    port_pin!("PE12", STM32_GPIO_CAP, PE12_FUNCS),
    port_pin!("PE13", STM32_GPIO_CAP, PE13_FUNCS),
    port_pin!("PE14", STM32_GPIO_CAP, PE14_FUNCS),
    port_pin!("PE15", STM32_GPIO_CAP, PE15_FUNCS),
    port_pin!("PF0", STM32_ANALOG_CAP, PF0_FUNCS),
    port_pin!("PF1", STM32_ANALOG_CAP, PF1_FUNCS),
    port_pin!("PF2", STM32_GPIO_CAP, PF2_FUNCS),
    port_pin!("PF9", STM32_GPIO_CAP, PF9_FUNCS),
    port_pin!("PF10", STM32_GPIO_CAP, PF10_FUNCS),
    port_pin!("PG10", STM32_GPIO_CAP, PG10_FUNCS),
];

pub static STM32G431: SiliconTarget = SiliconTarget {
    name: "stm32g431",
    family: "STM32",
    pins: STM32G431_PINS,
};
