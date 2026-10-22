//! The SPEAKER state (`eMAINMAPSTATES` 9): `CSTATEMGR_Speaker` (constructor 0x82550b40) and its
//! four `CSTATE_Speaker`s (`EnterGamePlay` 0x82550b98, created on the first entry only), each
//! with an `SFXObj_Speaker` (`CreateObject` 0x82549730), an `SFXCTL_SpeakerPhysics` and a
//! position controller: the public-address speakers of challenge venues, which carry the speech
//! and music through two `MultiSpeaker` delay voices.
//!
//! `AudioSystem::PlayMissionSFXImp` 0x824fedbc turns the multi-speaker mode on for the venues
//! `LoadCrowdsAndSpeakerSystem` 0x824e7d58 names (Demo and Best Trick challenges, one Jam). In
//! that mode `UpdateParams` 0x82557d58 builds the `MultiSpeaker` voices from `aud_general`'s
//! "multispeaker" collection, connects the submix (`ConnectSubmix` 0x825558a8) and attaches a
//! state to each of the first four speaker volumes of `speakers.ems` the emitter system reports
//! (`GetVolumes(eVolumeType_Speaker)`), detaching the rest; out of it the voices are freed
//! (`Cleanup` 0x82555a18). Every speaker volume also needs a game state only challenges set.
//!
//! No mission audio is ported, so the mode stays off and no `MultiSpeaker` exists: `UpdateParams`
//! is the base pass over detached states, and `ProcessUpdate` (0x82550c10, the base pass) finds
//! none attached. Not ported until missions exist: the multi-speaker branch of `UpdateParams`,
//! `ConnectSubmix`, `DisconnectSubmix` 0x82555948, `Cleanup`, `GetMultiSpeakerSubmix` 0x82550c18,
//! `CSTATE_Speaker` (`Attach` 0x825568d8, `Detach` 0x82551a80, its tuning getters
//! 0x82512f28-0x825134e8) and `SFXObj_Speaker` (`InitSFX` 0x82518150, `ProcessUpdate`
//! 0x82536a48, `Detach` 0x82536f68).

use crate::nicotine::MixMap;
use crate::player::pos3d::MicrophoneMan;
use crate::player::SfxCtx;
use crate::system::WorldAudioState;
use crate::vault::Vault;

/// `CSTATEMGR_Speaker` (0x28 bytes): without missions `mbIsMultiSpeakerMode` stays false and both
/// `mpMultiSpeaker` are null.
pub struct SpeakerManager;

impl SpeakerManager {
    #[must_use]
    pub fn new(_vault: &Vault, _map: Option<&MixMap>) -> Self {
        Self
    }

    /// `CSTATEMGR_Speaker::UpdateParams` 0x82557d58 out of multi-speaker mode with no voices: the
    /// base pass, which updates no detached state.
    pub fn update_params(&mut self, _t: f32, _world: &WorldAudioState, _mics: &MicrophoneMan, _ctx: &mut SfxCtx<'_>) {}

    /// `CSTATEMGR_Speaker::ProcessUpdate` 0x82550c10 (`CSTATEMGR_Base::ProcessUpdate`): no state is
    /// attached.
    pub fn process_update(&mut self, _ctx: &mut SfxCtx<'_>) {}
}
