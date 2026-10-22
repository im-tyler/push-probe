//! The CROWD state (`eMAINMAPSTATES` 7): `CSTATEMGR_Crowd` (constructor 0x82554120) and its five
//! `CSTATE_Crowd`s (shared `EnterGamePlay` 0x82550c60), each with an `SFXObj_Crowd` (TypeInfo
//! 0x00070000, `CreateObject` 0x82532298) whose `C_crowd_ambience` and `C_crowd_behaviour`
//! (`SK8_AEMS_Crowds.csi`) play the banks of `data\audio\crowds.big`.
//!
//! Crowds belong to challenges. `AudioSystem::PlayMissionSFXImp` 0x824fedbc loads their banks
//! (`LoadCrowdsData` 0x824fdcc8 disables the emitter swap first: the crowd banks load into the
//! emitters' swap memory, `GetCrowdsLoadStatus` 0x824f0818 follows them) and sets the
//! multi-speaker mode; `TriggerMissionMusicImp` 0x824ff234 and `AudioSystem::Update` (an arena
//! loaded with `mbPlayArenaOnLoad`) start them (`StartCrowds` 0x82557810 attaches a pod per crowd
//! volume of `crowds.ems`, or the arena pod); `CSTATEMGR_Crowd::PlayCrowdSound` 0x825573b0, which
//! `SFXObj_Contacts::CheckForScore` reaches through `AudioSystem::PlayCrowdSound`, cheers only
//! while a challenge key is set; `StopCrowds` 0x82554210 ends them.
//!
//! No challenge or mission audio is ported, so the challenge key stays 0, `mStartCrowds` false and
//! no state attaches: `UpdateParams` 0x825588d8 is the base pass over detached states and each
//! `SFXObj_Crowd::ProcessUpdate` 0x8254e4d8 returns at once. Not ported until missions exist:
//! `StartCrowds`, `AttachCrowdPod` 0x82557300, `PlayCrowdSound`, `StopCrowds`, `LoadCrowdsData`,
//! `GetNearestCrowdPodInfo` 0x824f0298, `CacheCrowdPodInfos` 0x824fd6c0 and the `SFXObj_Crowd`
//! object (`InitSFX` 0x82547170, `StartCrowd` 0x82532428, `HandleAmbience` 0x82547270,
//! `HandleBehaviour` 0x82547608, `PlayCheer` 0x82532720, `PlayOhh` 0x82532860).

use crate::nicotine::MixMap;
use crate::player::pos3d::MicrophoneMan;
use crate::player::SfxCtx;
use crate::system::WorldAudioState;
use crate::vault::Vault;

/// `CSTATEMGR_Crowd` (0x48 bytes): without a challenge its `mChallengeKey` stays 0 and
/// `mStartCrowds` false.
pub struct CrowdManager;

impl CrowdManager {
    #[must_use]
    pub fn new(_vault: &Vault, _map: Option<&MixMap>) -> Self {
        Self
    }

    /// `CSTATEMGR_Crowd::UpdateParams` 0x825588d8: `StartCrowds` while `mStartCrowds` is set,
    /// then the base pass, which updates no detached state.
    pub fn update_params(&mut self, _t: f32, _world: &WorldAudioState, _mics: &MicrophoneMan, _ctx: &mut SfxCtx<'_>) {}

    /// `CSTATEMGR_Base::ProcessUpdate`: every `SFXObj_Crowd` returns while its state is detached.
    pub fn process_update(&mut self, _ctx: &mut SfxCtx<'_>) {}
}
