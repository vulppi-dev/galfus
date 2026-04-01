#[cfg(not(target_arch = "wasm32"))]
use gilrs::Gilrs;

#[derive(Debug, Default)]
pub struct PlatformGamepadBackendState {
    #[cfg(not(target_arch = "wasm32"))]
    pub gilrs: Option<Gilrs>,
}

impl PlatformGamepadBackendState {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let gilrs = match Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(error) => {
                log::warn!("Failed to initialize gamepad support: {:?}", error);
                None
            }
        };

        Self {
            #[cfg(not(target_arch = "wasm32"))]
            gilrs,
        }
    }
}
