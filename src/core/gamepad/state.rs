#[cfg(not(feature = "wasm"))]
use crate::core::platform::gilrs;

#[derive(Debug, Default)]
pub struct GamepadBackendState {
    #[cfg(not(feature = "wasm"))]
    pub gilrs: Option<gilrs::Gilrs>,
}

impl GamepadBackendState {
    pub fn new() -> Self {
        #[cfg(not(feature = "wasm"))]
        let gilrs = match gilrs::Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(e) => {
                log::warn!("Failed to initialize gamepad support: {:?}", e);
                None
            }
        };

        Self {
            #[cfg(not(feature = "wasm"))]
            gilrs,
        }
    }
}
