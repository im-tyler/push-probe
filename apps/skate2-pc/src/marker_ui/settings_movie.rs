//! Options screen120: FrontEndState_MainMenu::OnMenuSelectGame82681604
//! pushes state73; Options::OnEnter8265c468 opens options/options.swf.
//! Its fingerprinted CreateOptions/SetSelection/UpdateItemValue scripts own
//! placement and controls. Only the two PC setting definitions are new.
use super::{
    assets::Assets,
    clips::{Child, Clip},
    edit_skater_movie,
    letterbox::LetterBox,
    localizer, map_movie,
    pause_movie::child,
};
use crate::gpu_renderer::RenderResult;
use skate2_core::ui::{
    apt_timeline::{DisplayEntry, PlacementIdentity},
    options::{FrameRateCap, Page, Row, Snapshot},
    edit_skater::EditSkaterRow,
};
pub(super) struct SettingsMovie {
    pub root: Clip,
    /// The LetterBox of Options opened in game; over the main menu Options
    /// drives the main menu's.
    pub letterbox: LetterBox,
    pub state: Snapshot,
    /// Opened over the main menu (`mFEState == eFE_STATE_IN_FE`).
    in_front_end: bool,
    /// `OnEnter`'s `OpenScreen(120)` waits for the screen manager to link the screen.
    pending: bool,
    /// `OutroComplete`'s `DestroyScreen`: the next tick's `ProcessDestroyedScreens` removes it.
    destroy: bool,
    /// The outro's `OutroComplete` since the last take.
    outro_complete: bool,
    /// The intro's `IntroComplete` since the last take.
    intro_complete: bool,
}
impl SettingsMovie {
    pub fn new(assets: &Assets) -> RenderResult<Self> {
        let mut root = Clip::new(assets, 28, 0)?;
        root.visible = false;
        Ok(Self {
            root,
            letterbox: LetterBox::new(assets)?,
            state: Snapshot::default(),
            in_front_end: false,
            pending: false,
            destroy: false,
            outro_complete: false,
            intro_complete: false,
        })
    }
    /// The state waits for its screen.
    pub fn pending(&self) -> bool {
        self.pending
    }
    /// FE state 73 is on the stack, or the music player (30) Options pushed over it.
    pub fn on_stack(&self) -> bool {
        self.state.page != Page::Hidden || self.state.music_player
    }
    /// Whether the main menu's letterbox is the one on screen.
    pub fn in_front_end(&self) -> bool {
        self.in_front_end
    }
    /// The screen's `IntroComplete` since the last call.
    pub fn take_intro_complete(&mut self) -> bool {
        std::mem::take(&mut self.intro_complete)
    }
    /// The screen's `OutroComplete` since the last call.
    pub fn take_outro_complete(&mut self) -> bool {
        std::mem::take(&mut self.outro_complete)
    }
    pub fn update(
        &mut self,
        assets: &Assets,
        state: Snapshot,
        mut main_letterbox: Option<&mut LetterBox>,
    ) -> RenderResult<()> {
        if self.state == state {
            return Ok(());
        }
        let previous = std::mem::replace(&mut self.state, state);
        if previous.page == Page::Hidden && state.page != Page::Hidden {
            // OnEnter 0x8265c468's OpenScreen(120): the screen manager links the screen later.
            // LetterBox is global in the source: over the main menu its bars and title are
            // already up.
            if !previous.music_player {
                self.in_front_end = main_letterbox.is_some();
            }
            self.pending = true;
        } else if state.page == Page::Hidden && previous.page != Page::Hidden {
            // OnLeave 0x8266af70's CloseScreen plays the outro; outside the front end it sends
            // SetState2 first: EnableLetterBoxInfo(false), EnableLetterBox(false).
            if !std::mem::take(&mut self.pending) {
                self.root.goto(assets, "outro")?;
            }
            if !self.in_front_end {
                self.letterbox.enable_info(assets, false)?;
                self.letterbox.enable(assets, false)?;
            }
        } else if state.page != Page::Hidden && !self.pending {
            self.layout(assets, &previous, main_letterbox.as_deref_mut())?;
        }
        self.callbacks(assets, main_letterbox)
    }
    /// The screen manager linked the screen.
    pub fn link(&mut self, assets: &Assets, mut main_letterbox: Option<&mut LetterBox>) -> RenderResult<()> {
        if std::mem::take(&mut self.pending) {
            self.construct(assets, main_letterbox.as_deref_mut())?;
        }
        self.callbacks(assets, main_letterbox)
    }
    /// The screen manager links the screen: the constructor's `ShowLetterbox` (03ea:
    /// EnableLetterBox(true) and EnableLetterBoxInfo(true), which animate only what is hidden),
    /// its `intro`, then `LoadMainMenu` 0x826714d0's `SetState(1)` lays out the page.
    fn construct(&mut self, assets: &Assets, mut main_letterbox: Option<&mut LetterBox>) -> RenderResult<()> {
        self.root = Clip::new(assets, 28, 0)?;
        self.destroy = false;
        for name in ["dummy0", "dummy1", "dummy2"] {
            self.root.named(&[name])?.visible = false;
        }
        self.root.goto(assets, "intro")?;
        let letterbox = options_letterbox(&mut self.letterbox, main_letterbox.as_deref_mut(), self.in_front_end);
        letterbox.enable(assets, true)?;
        letterbox.enable_info(assets, true)?;
        self.layout(assets, &Snapshot::default(), main_letterbox)
    }
    /// A new page: SetState1 (and the subpage switches in OnMenuSelect 0x826871a0 and
    /// LoadMainMenu) retitle the letterbox, then `CreateOptions`; the same page takes
    /// `SetSelection` and SetState50.
    fn layout(&mut self, assets: &Assets, previous: &Snapshot, main_letterbox: Option<&mut LetterBox>) -> RenderResult<()> {
        let state = self.state;
        let letterbox = options_letterbox(&mut self.letterbox, main_letterbox, self.in_front_end);
        let rows = state
            .rows()
            .iter()
            .map(|&row| options_row(assets, &state, row))
            .collect::<RenderResult<Vec<_>>>()?;
        if previous.page != state.page {
            let title = state.title();
            let title = if title.starts_with("ID_") { assets.localize(title)? } else { title.into() };
            letterbox.set_title(&title)?;
            // GetButtonHelpTextA 0x82644128 names A only over `option` rows.
            let help: &[(&str, &str)] = if rows.first().is_some_and(|row| row.kind == "option") {
                &[("B", "ID_COMMON_BACK"), ("A", "ID_COMMON_SELECT")]
            } else {
                &[("B", "ID_COMMON_BACK")]
            };
            map_movie::configure_button_items(
                letterbox.clip.named(&[
                    "mcLetterbox",
                    "mInfo",
                    "mcBottomMenu",
                    "mButtonMenu",
                ])?,
                assets,
                help,
                0xffff_ffec,
            )?;
            // options.CreateOptions0008e5..ab1: first position is dummy0;
            // subsequent y = previous.y + previous.GetHeight(), no CAC gap.
            let mut matrix = child(&mut self.root, "dummy0")?.entry.matrix;
            self.root.children.retain(|_, c| {
                c.entry
                    .name
                    .as_deref()
                    .is_some_and(|n| n.starts_with("dummy"))
            });
            for (index, row) in rows.iter().enumerate() {
                let (asset, export, height) = match row.kind.as_str() {
                    "option" => (20, "HighlightOptionComponent", 36.), // GetHeight002272
                    "slider" => (21, "HighlightSliderComponent", 72.), // prototype mHeight
                    _ => (18, "HighlightSelectorComponent", 58.),      // GetHeight001868
                };
                let character = assets.movies[asset]
                    .movie
                    .exports
                    .iter()
                    .find(|e| e.name == export)
                    .ok_or("original Options control export absent")?
                    .character_id;
                let mut clip = Clip::new(assets, asset, character)?;
                edit_skater_movie::highlight(&mut clip, assets, row, index == state.selected)?;
                self.root.children.insert(
                    index as i32 + 100,
                    Child {
                        entry: DisplayEntry {
                            identity: PlacementIdentity {
                                character,
                                frame: 0,
                            },
                            matrix,
                            colour_scale: u32::MAX,
                            colour_bias: 0,
                            ratio: 0.,
                            name: Some(format!("mItem{index}")),
                            clip_depth: -1,
                            events: Vec::new(),
                            blend_mode: None,
                        },
                        clip,
                    },
                );
                matrix.ty += height;
            }
        } else {
            for (index, row) in rows.iter().enumerate() {
                let clip = self.root.named(&[&format!("mItem{index}")])?;
                if previous.selected != state.selected {
                    edit_skater_movie::highlight(clip, assets, row, index == state.selected)?;
                } else if row.kind == "selector" {
                    // SetState50 refreshes the value without replaying selection.
                    edit_skater_movie::selector_subtitle(clip, assets, &row.subtitle)?;
                } else if row.kind == "slider" {
                    edit_skater_movie::slider_value(clip, assets, &row.subtitle, row.value, index == state.selected)?;
                }
            }
        }
        Ok(())
    }
    /// `LetterBox.EnableLetterBox`, called by a screen over Options.
    pub fn enable_letterbox(&mut self, assets: &Assets, enable: bool, main_letterbox: Option<&mut LetterBox>) -> RenderResult<()> {
        options_letterbox(&mut self.letterbox, main_letterbox, self.in_front_end).enable(assets, enable)
    }
    /// Over the main menu, the main menu ticks the shared letterbox.
    pub fn tick(&mut self, assets: &Assets, main_letterbox: Option<&mut LetterBox>) -> RenderResult<()> {
        // ScreenManager.Update's ProcessDestroyedScreens removes the screen the tick after its
        // OutroComplete.
        if std::mem::take(&mut self.destroy) {
            self.root.visible = false;
        }
        if self.root.visible {
            self.root.tick(assets)?;
        }
        // The letterbox outlives the screen until its own outro hides it.
        if !self.in_front_end && self.letterbox.clip.visible {
            self.letterbox.tick(assets)?;
        }
        self.callbacks(assets, main_letterbox)
    }
    fn callbacks(&mut self, assets: &Assets, mut main_letterbox: Option<&mut LetterBox>) -> RenderResult<()> {
        let mut calls = Vec::new();
        self.root.take_calls(&mut calls);
        for call in calls {
            match call {
                // ScreenManager.EnableDimmer, a method screenmanager.apt never defines; the FE
                // states' mbDimmer drives the dimmer (StateManagerFrontEnd::EnableDimmer 0x82656a30).
                "OptionsDimmerOn" => {}
                // Frame 4: EnableLetterBoxInfo(true), then ScreenManager.IntroComplete.
                "OptionsIntroComplete" => {
                    options_letterbox(&mut self.letterbox, main_letterbox.as_deref_mut(), self.in_front_end)
                        .enable_info(assets, true)?;
                    self.intro_complete = true;
                }
                // Frame 20: ScreenManager.OutroComplete, whose DestroyScreen queues the removal.
                "OptionsOutroComplete" => {
                    self.destroy = true;
                    self.outro_complete = true;
                }
                _ => return Err(format!("untranslated Options callback {call}").into()),
            }
        }
        Ok(())
    }
}

/// `CreateOptions`/`UpdateItemValue`: the row's title, then a slider's `ID_COMMON_PERCENT` of
/// `GetIntegerValue * 10` or a selector's ON/OFF, or the PC Frame Rate Cap's value.
fn options_row(assets: &Assets, state: &Snapshot, row: Row) -> RenderResult<EditSkaterRow> {
    let title = row.title();
    let title = if title.starts_with("ID_") { assets.localize(title)? } else { title.into() };
    let volume = match row {
        Row::SfxVolume => Some(state.volumes[0]),
        Row::DialogueVolume => Some(state.volumes[1]),
        Row::MusicVolume => Some(state.volumes[2]),
        _ => None,
    };
    let (subtitle, value) = if let Some(value) = volume {
        let percent = localizer::localized_format_string(assets, "ID_COMMON_PERCENT", &[localizer::AptArgument::Integer(value * 10)])?;
        (String::from_utf16_lossy(&percent), value)
    } else if row.kind() == "selector" {
        let on_off = |on: bool| if on { "ID_COMMON_ON" } else { "ID_COMMON_OFF" };
        let text = match row {
            Row::PostFx => on_off(state.values.postfx),
            Row::Fisheye => on_off(state.values.fisheye),
            Row::FpsCounter => on_off(state.values.fps_counter),
            Row::Fullscreen => on_off(state.values.fullscreen),
            Row::Subtitles => on_off(state.subtitles),
            Row::GrindTransparency => on_off(state.grind_transparency),
            Row::MiniMap => on_off(state.mini_map),
            Row::Hud => on_off(state.hud),
            Row::ShowLobbyChat => on_off(state.show_lobby_chat),
            // GetStringValue 0x82643d48: ID_GAMESETTINGS_CAMERA_HIGH for 1, _LOW for 0.
            Row::CameraAngle => if state.camera_high { "ID_GAMESETTINGS_CAMERA_HIGH" } else { "ID_GAMESETTINGS_CAMERA_LOW" },
            _ => match state.values.frame_rate_cap {
                FrameRateCap::Unlimited => "Unlimited",
                FrameRateCap::Fps60 => "60 (Vanilla)",
                FrameRateCap::Fps120 => "120",
                FrameRateCap::RefreshRate => "Refresh Rate",
            },
        };
        (if text.starts_with("ID_") { assets.localize(text)? } else { text.into() }, 0)
    } else {
        (String::new(), 0)
    };
    Ok(EditSkaterRow { kind: row.kind().into(), title, subtitle, value, ..Default::default() })
}

/// Options' LetterBox: the main menu's over the front end, else its own.
fn options_letterbox<'a>(
    own: &'a mut LetterBox,
    main: Option<&'a mut LetterBox>,
    in_front_end: bool,
) -> &'a mut LetterBox {
    match main {
        Some(main) if in_front_end => main,
        _ => own,
    }
}

pub(super) fn action(clip: &mut Clip, offset: usize) -> RenderResult<()> {
    match offset {
        0x378 | 0x3a4 => clip.playback.stop(), // constructor/class translated above
        0xc38 => clip.calls.push("OptionsDimmerOn"),
        0xc54 => {
            clip.playback.stop();
            clip.calls.push("OptionsIntroComplete");
        }
        0xc7c => {
            clip.playback.stop();
            clip.calls.push("OptionsOutroComplete");
        }
        _ => return Err(format!("untranslated Options APT action {offset:#x}").into()),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use skate2_formats::mount::{MountSet, MountedBig, SKATE2_ARCHIVES};

    #[test]
    #[ignore = "requires external SKATE2_GAME_ROOT"]
    fn display_preferences_original_options_layout_and_lifecycle() {
        let game = std::path::PathBuf::from(std::env::var_os("SKATE2_GAME_ROOT").unwrap());
        let mut mounts = MountSet::new();
        for spec in SKATE2_ARCHIVES.iter().take(4) {
            mounts
                .push(MountedBig::open(spec.path_under(&game), spec.use_external_header).unwrap());
        }
        let assets = Assets::load(&mounts).unwrap();
        let mut wall = super::super::main_movie::MainMovie::new(&assets).unwrap();
        wall.update(&assets, true, 5, false).unwrap();
        wall.link(&assets).unwrap();
        for _ in 0..90 {
            wall.tick(&assets).unwrap();
        }
        // The main menu's letterbox plays its entrance and title once.
        assert_eq!(
            std::mem::take(&mut wall.letterbox.sounds),
            ["FE_SOUND_LETTERBOX_IN", "FE_SOUND_PAGETITLE_FLICKER"]
        );
        for front_end in [true, false] {
            let mut main = front_end.then(|| wall.letterbox.clone());
            let mut movie = SettingsMovie::new(&assets).unwrap();
            // Loading saved preferences while hidden must not start an outro.
            movie.update(&assets, Snapshot { values: skate2_core::ui::options::PcDisplaySettings {
                postfx: false, fisheye: false, ..Default::default() }, ..Default::default() }, main.as_mut()).unwrap();
            assert!(!movie.pending() && !movie.root.visible);
            assert_eq!(movie.root.playback.timeline().frame(), Some(0));
            let mut state = Snapshot {
                page: Page::Options,
                ..Default::default()
            };
            movie.update(&assets, state, main.as_mut()).unwrap();
            // Nothing shows before the screen manager links the screen.
            assert!(movie.pending() && !movie.root.visible);
            movie.link(&assets, main.as_mut()).unwrap();
            tick(&mut movie, &mut main, &assets, 90);
            // ShowLetterbox and frame 4 animate only a hidden letterbox and title.
            let entrance: &[&str] = if front_end {
                &[]
            } else {
                &["FE_SOUND_LETTERBOX_IN", "FE_SOUND_PAGETITLE_FLICKER"]
            };
            assert_eq!(std::mem::take(&mut shown(&mut movie, &mut main).sounds), entrance);
            assert!(movie.root.visible);
            assert_eq!(movie.root.asset, 28);
            let anchor = child(&mut movie.root, "dummy0").unwrap().entry.matrix;
            let first = child(&mut movie.root, "mItem0").unwrap();
            assert_eq!(first.entry.matrix, anchor);
            assert_eq!(first.clip.asset, 20);
            assert!((anchor.tx - 436.85).abs() < 0.001 && (anchor.ty - 199.15).abs() < 0.001);
            let help_x = child(
                shown(&mut movie, &mut main)
                    .clip
                    .named(&["mcLetterbox", "mInfo", "mcBottomMenu", "mButtonMenu"])
                    .unwrap(),
                "mText0",
            )
            .unwrap()
            .entry
            .matrix
            .tx;
            // Repeated page changes must never accumulate button-help drift.
            for _ in 0..3 {
                state.page = Page::Display;
                state.selected = 0;
                movie.update(&assets, state, main.as_mut()).unwrap();
                tick(&mut movie, &mut main, &assets, 90);
                assert_eq!(
                    child(&mut movie.root, "mItem1").unwrap().entry.matrix.ty,
                    anchor.ty + 58.
                );
                for selected in [0, 1, 0] {
                    state.selected = selected;
                    movie.update(&assets, state, main.as_mut()).unwrap();
                    tick(&mut movie, &mut main, &assets, 30);
                    for index in 0..2 {
                        let item = movie.root.named(&[&format!("mItem{index}")]).unwrap();
                        for arrow in ["mArrowleft", "mArrowright"] {
                            assert_eq!(item.named(&[arrow]).unwrap().visible, index == selected);
                        }
                    }
                }
                let frame = movie
                    .root
                    .named(&["mItem0"])
                    .unwrap()
                    .playback
                    .timeline()
                    .frame();
                state.values.postfx = !state.values.postfx;
                movie.update(&assets, state, main.as_mut()).unwrap();
                assert_eq!(
                    movie
                        .root
                        .named(&["mItem0"])
                        .unwrap()
                        .playback
                        .timeline()
                        .frame(),
                    frame
                );
                let units = movie
                    .root
                    .named(&["mItem0", "mSubtitle", "TEXTLABEL"])
                    .unwrap()
                    .text
                    .as_ref()
                    .unwrap();
                assert_eq!(
                    String::from_utf16(units).unwrap(),
                    assets
                        .localize(if state.values.postfx {
                            "ID_COMMON_ON"
                        } else {
                            "ID_COMMON_OFF"
                        })
                        .unwrap()
                );
                // A game's Video Settings: Camera Angle, Grind Transparency, then Subtitles with
                // the original titles and values.
                state.page = Page::Video;
                state.video = skate2_core::ui::options::VideoScreen::Game;
                let text = |clip: &mut Clip, path: &[&str]| String::from_utf16(clip.named(path).unwrap().text.as_ref().unwrap()).unwrap();
                for on in [false, true] {
                    state.subtitles = on;
                    state.camera_high = on;
                    movie.update(&assets, state, main.as_mut()).unwrap();
                    let item = movie.root.named(&["mItem2"]).unwrap();
                    assert_eq!(text(&mut *item, &["mTitle", "TEXTLABEL"]), assets.localize("ID_GAMESETTINGS_SUBTITLE_TOGGLE").unwrap());
                    assert_eq!(text(&mut *item, &["mSubtitle", "TEXTLABEL"]), assets.localize(if on { "ID_COMMON_ON" } else { "ID_COMMON_OFF" }).unwrap());
                    let camera = movie.root.named(&["mItem0"]).unwrap();
                    assert_eq!(text(&mut *camera, &["mSubtitle", "TEXTLABEL"]),
                        assets.localize(if on { "ID_GAMESETTINGS_CAMERA_HIGH" } else { "ID_GAMESETTINGS_CAMERA_LOW" }).unwrap());
                }
                state.subtitles = false;
                state.camera_high = false;
                state.page = Page::Display;
                movie.update(&assets, state, main.as_mut()).unwrap();
                // Every Frame Rate Cap value has its glyphs in the selector's font.
                for (frame_rate_cap, label) in [
                    (FrameRateCap::Unlimited, "Unlimited"),
                    (FrameRateCap::Fps60, "60 (Vanilla)"),
                    (FrameRateCap::Fps120, "120"),
                    (FrameRateCap::RefreshRate, "Refresh Rate"),
                ] {
                    state.values.frame_rate_cap = frame_rate_cap;
                    movie.update(&assets, state, main.as_mut()).unwrap();
                    let field = movie.root.named(&["mItem2", "mSubtitle", "TEXTLABEL"]).unwrap();
                    assert_eq!(String::from_utf16(field.text.as_ref().unwrap()).unwrap(), label);
                    let skate2_formats::apt::Character::Text(text) = field.definition(&assets).unwrap() else {
                        panic!("selector subtitle is not text");
                    };
                    let Some(skate2_formats::apt::Character::Font { name, .. }) =
                        assets.movies[field.asset].movie.characters[text.font_id as usize].as_ref()
                    else {
                        panic!("selector subtitle font absent");
                    };
                    let font = assets.apt_font(name).unwrap();
                    assert!(label.encode_utf16().all(|unit| font.metrics.mapped_glyph(unit).is_some()), "{label}");
                }
                // Render the real retained movie at normal and ultrawide sizes.
                let letterbox = shown(&mut movie, &mut main).clip.clone();
                for size in [(1280, 720), (2560, 720)] {
                    let mut batches = Vec::new();
                    for clip in [&movie.root, &letterbox] {
                        super::super::flatten_projected(
                            &assets,
                            clip,
                            skate2_core::ui::apt_timeline::IDENTITY,
                            super::super::Colour::default(),
                            size,
                            &mut batches,
                            true,
                            None,
                        )
                        .unwrap();
                    }
                    assert!(!batches.is_empty());
                    assert!(
                        batches
                            .iter()
                            .flat_map(|b| &b.vertices)
                            .flatten()
                            .all(|x| x.is_finite())
                    );
                }
                state.page = Page::Options;
                movie.update(&assets, state, main.as_mut()).unwrap();
                let x = child(
                    shown(&mut movie, &mut main)
                        .clip
                        .named(&["mcLetterbox", "mInfo", "mcBottomMenu", "mButtonMenu"])
                        .unwrap(),
                    "mText0",
                )
                .unwrap()
                .entry
                .matrix
                .tx;
                assert_eq!(x, help_x);
            }
            // Subpages only retitle the letterbox.
            assert!(shown(&mut movie, &mut main).sounds.is_empty());
            state.page = Page::Hidden;
            movie.update(&assets, state, main.as_mut()).unwrap();
            assert!(movie.root.visible);
            tick(&mut movie, &mut main, &assets, 90);
            assert!(!movie.root.visible);
            // SetState2 takes the in-game letterbox out; the main menu keeps its own.
            let exit: &[&str] = if front_end { &[] } else { &["FE_SOUND_LETTERBOX_OUT"] };
            assert_eq!(std::mem::take(&mut shown(&mut movie, &mut main).sounds), exit);
            assert!(movie.take_outro_complete());
        }
    }

    /// MarkerUi's step order: the main menu ticks the shared letterbox first.
    fn tick(movie: &mut SettingsMovie, main: &mut Option<LetterBox>, assets: &Assets, steps: usize) {
        for _ in 0..steps {
            if let Some(main) = main.as_mut() {
                main.tick(assets).unwrap();
            }
            movie.tick(assets, main.as_mut()).unwrap();
        }
    }

    fn shown<'a>(movie: &'a mut SettingsMovie, main: &'a mut Option<LetterBox>) -> &'a mut LetterBox {
        options_letterbox(&mut movie.letterbox, main.as_mut(), movie.in_front_end)
    }
}
