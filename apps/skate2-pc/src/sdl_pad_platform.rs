//! The pads where Windows' pad API does not exist (Linux): every pad through SDL3
//! (`crate::sdl_pads`), each reported as `XInputGetState` and `XInputGetCapabilities` report a
//! wired Xbox 360 gamepad. Native mapping, polling cadence and caches live in core.
use crate::sdl_pads::SdlPads;
use skate2_core::{
    PadCapabilitiesResult, PadPlatform, PadStateResult, PadVibration, XboxPadSample,
};

/// `ERROR_DEVICE_NOT_CONNECTED`: what `XInputGetState` returns for a slot with no pad.
const DEVICE_NOT_CONNECTED: u32 = 1167;
/// `XINPUT_FLAG_GAMEPAD`, the capabilities the game asks for.
const FLAG_GAMEPAD: u32 = 1;

pub(crate) struct SdlPadPlatform {
    /// Explicit diagnostic replay sample for controller zero; None reads SDL.
    pub(crate) demo_sample: Option<XboxPadSample>,
    sdl: Option<SdlPads>,
}
impl SdlPadPlatform {
    pub(crate) fn new(demo_sample: Option<XboxPadSample>, sdl: Option<SdlPads>) -> Self {
        Self { demo_sample, sdl }
    }
}
#[derive(Debug)]
pub(crate) struct UnsupportedCapabilitiesFlags(u32);
impl std::fmt::Display for UnsupportedCapabilitiesFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "unsupported native controller capability flags {}",
            self.0
        )
    }
}
impl std::error::Error for UnsupportedCapabilitiesFlags {}

impl PadPlatform for SdlPadPlatform {
    type Error = UnsupportedCapabilitiesFlags;
    fn get_state(&mut self, index: u32) -> Result<PadStateResult, Self::Error> {
        if index == 0 {
            if let Some(sample) = self.demo_sample {
                return Ok(PadStateResult {
                    status: 0,
                    state: Some((7, sample)),
                });
            }
        }
        let Some((packet, sample)) = self.sdl.as_mut().and_then(|sdl| sdl.claim(index)) else {
            return Ok(PadStateResult {
                status: DEVICE_NOT_CONNECTED,
                state: None,
            });
        };
        if index == 0 && std::env::var_os("SKATE2_TRACE_XINPUT").is_some() {
            eprintln!("sdl slot=0 packet={packet} sample={sample:?}");
        }
        Ok(PadStateResult {
            status: 0,
            state: Some((packet, sample)),
        })
    }
    fn get_capabilities(
        &mut self,
        index: u32,
        flags: u32,
    ) -> Result<PadCapabilitiesResult, Self::Error> {
        if flags != FLAG_GAMEPAD {
            return Err(UnsupportedCapabilitiesFlags(flags));
        }
        // A diagnostic sample advertises the original-data pad corpus's controller-0
        // capabilities, as on Windows.
        if index == 0 && self.demo_sample.is_some() {
            return Ok(PadCapabilitiesResult {
                status: 0,
                device_type: Some(1),
                subtype: Some(0),
                flags: Some(0),
            });
        }
        // As DS4Windows showed its pads on Windows: type and subtype gamepad, no wireless flag.
        Ok(if self.sdl.as_mut().and_then(|sdl| sdl.claim(index)).is_some() {
            PadCapabilitiesResult {
                status: 0,
                device_type: Some(1),
                subtype: Some(1),
                flags: Some(0),
            }
        } else {
            PadCapabilitiesResult {
                status: DEVICE_NOT_CONNECTED,
                device_type: None,
                subtype: None,
                flags: None,
            }
        })
    }
    fn set_state(&mut self, index: u32, vibration: PadVibration) -> Result<u32, Self::Error> {
        Ok(if self.sdl.as_mut().is_some_and(|sdl| sdl.vibrate(index, vibration)) {
            0
        } else {
            DEVICE_NOT_CONNECTED
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_without_sdl_read_as_xinput_empty_slots() {
        let mut platform = SdlPadPlatform::new(None, None);
        for index in 0..4 {
            let state = platform.get_state(index).unwrap();
            assert_eq!((state.status, state.state.is_some()), (DEVICE_NOT_CONNECTED, false));
            let caps = platform.get_capabilities(index, FLAG_GAMEPAD).unwrap();
            assert_eq!(caps.status, DEVICE_NOT_CONNECTED);
            assert!(caps.device_type.is_none() && caps.subtype.is_none() && caps.flags.is_none());
            let vibration = PadVibration { left: 0, right: 0 };
            assert_eq!(platform.set_state(index, vibration).unwrap(), DEVICE_NOT_CONNECTED);
        }
        assert!(platform.get_capabilities(0, 0).is_err());
    }
}
