//! STM32G431 48-pin package variants against DS12589 Rev 6, Table 12.

use crate::silicon::*;

#[test]
fn c_package_variants_add_their_extra_gpios() {
    let bonded = |chip: &str, pin: &str| find_target(chip).unwrap().find_pin(pin).is_some();
    assert!(bonded("stm32g431cbu", "PC10"));
    assert!(bonded("stm32g431cbu", "PC4"));
    assert!(!bonded("stm32g431cby", "PC10"));
    assert!(bonded("stm32g431cby", "PC11"));
    assert!(!bonded("stm32g431cbt", "PC11"));
    assert!(find_target("stm32g431cbx").is_none());
}
