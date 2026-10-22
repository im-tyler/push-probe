//! Moving visual inventory using the session's registered stream metadata.
//! Native focus selection is reused. Each generation's high collection loads
//! and unloads drive the PROXY registrations (`proxy_world`); full processed
//! Presentation registration remains separate work. The mission hulls the
//! simulation's hull loaders registered join each generation
//! (`hull_presentation`).
use crate::{
    gpu_renderer::{GpuTextureUploader, RenderResult},
    proxy_texture_world::ProxyTextures,
    proxy_world::SharedProxyWorld,
};
use skate2_core::{
    game_stream::StreamKind,
    presentation::{HullPresentation, PresentationScene},
    streaming::{CollectionRequestKind, StreamFocusTransition},
    texture::TextureScene,
    world_stream_catalog::WorldStreamCatalog,
};
use skate2_formats::{
    mount::MountSet,
    stream::{
        CollectionMap, CollectionSpace, FocusCollection, TEXTURE_FOCUS_RADIUS, WORLD_FOCUS_RADIUS,
    },
    stream_xml::StreamXml,
};
use std::{
    collections::BTreeSet,
    sync::{Arc, mpsc},
    thread,
};

struct FocusSource {
    map: Arc<CollectionMap>,
    space: Arc<CollectionSpace>,
    xml: Option<Arc<StreamXml>>,
    radius: f32,
}
impl FocusSource {
    fn from_registered(
        catalog: &WorldStreamCatalog,
        seed: u64,
        kind: StreamKind,
        radius: f32,
    ) -> RenderResult<Self> {
        let collection = catalog
            .collection(seed)?
            .ok_or("visual seed collection not registered")?;
        let file = catalog
            .file(&collection.owner)?
            .ok_or("visual seed stream not registered")?;
        if file.kind != kind {
            return Err("visual seed stream kind mismatch".into());
        }
        let map = file.collection_map.ok_or("visual stream XSM absent")?;
        let space = file.collection_space.ok_or("visual stream XSS absent")?;
        let xml = file
            .stream_xml
            .as_ref()
            .filter(|bytes| !bytes.is_empty())
            .map(|bytes| StreamXml::parse(bytes, kind.suffix(), &space).map(Arc::new))
            .transpose()?;
        Ok(Self {
            map,
            space,
            xml,
            radius,
        })
    }
    fn select(&self, position: [f32; 3], xml_enabled: bool) -> Vec<FocusCollection> {
        // FindCollections824df068 uses a fixed capacity of16 for Pres/Tex.
        // The simulation-specific cvar is not a presentation cap.
        if xml_enabled {
            if let Some(xml) = &self.xml {
                return xml.focus_collections(&self.space, position, self.radius, 16);
            }
        }
        self.space.focus_collections(position, self.radius, 16)
    }
}

pub(crate) type PreparedRenderState = (
    skate2_core::render::NativeRenderQueue,
    crate::render_inspection::DiagnosticTechniques,
);
/// A generation's owners, render state and the proxy revision its scene holds
/// (none before activation).
type Loaded = Result<
    (
        PresentationScene,
        TextureScene,
        PreparedRenderState,
        Option<u64>,
    ),
    String,
>;

/// One generation's high collection registrations and their proxies
/// (`NotifyCollectionPresDReg` 0x82d580d0 / `NotifyCollectionPresReg`
/// 0x82d5c680): the executed unloads re-register their proxies, then the
/// executed loads deregister theirs, in the request queue's order
/// (`move_runtime_presentation_selection` drains every unload before any
/// load). The scene then holds the registered proxies; returns their revision.
fn register_proxies(
    proxy: &SharedProxyWorld,
    active: bool,
    transition: Option<&StreamFocusTransition>,
    scene: &mut PresentationScene,
) -> RenderResult<Option<u64>> {
    if !active {
        return Ok(None);
    }
    let mut owner = proxy.lock().map_err(|_| "PROXY world lock poisoned")?;
    if let Some(transition) = transition {
        for request in &transition.requests {
            if matches!(request.kind, CollectionRequestKind::Unload) {
                owner.high_deregistered(request.id)?;
            }
        }
        for request in &transition.requests {
            if matches!(request.kind, CollectionRequestKind::Load) {
                owner.high_registered(request.id)?;
            }
        }
    }
    scene.proxies = owner.registered_collections();
    Ok(Some(owner.revision()?))
}

pub(crate) struct CityStreaming {
    pub(crate) mounts: Arc<MountSet>,
    pub(crate) catalog: WorldStreamCatalog,
    proxy: SharedProxyWorld,
    /// Proxy revision of the installed scene; none before activation.
    proxy_revision: Option<u64>,
    sources: Option<(FocusSource, FocusSource)>,
    pending: Option<mpsc::Receiver<Loaded>>,
}
impl CityStreaming {
    pub(crate) fn is_pending(&self) -> bool { self.pending.is_some() }
    pub(crate) fn new(
        mounts: Arc<MountSet>,
        catalog: WorldStreamCatalog,
        proxy: SharedProxyWorld,
    ) -> Self {
        Self {
            mounts,
            catalog,
            proxy,
            proxy_revision: None,
            sources: None,
            pending: None,
        }
    }

    /// Loads the PROXY collections in `mProxyCollectionID` order (texture
    /// uploads need the renderer), activates them (`SyncActivateProxyWorld`
    /// 0x824d9ef8), then registers `scene`'s resident high collections, whose
    /// proxies deregister, and hands `scene` the registered proxies.
    /// The presentation stream for the session's `IsAreaLoaded`, with the high collections
    /// `scene` registered; none before the first focus move picked the stream.
    pub(crate) fn presentation_area(
        &self,
        scene: &PresentationScene,
    ) -> Option<skate2_core::streaming::area_loaded::PresentationStreamArea> {
        let (pres, _) = self.sources.as_ref()?;
        Some(skate2_core::streaming::area_loaded::PresentationStreamArea {
            space: Arc::clone(&pres.space),
            xml: pres.xml.clone(),
            radius: pres.radius,
            resident: scene.collections.iter().map(|collection| collection.id).collect(),
        })
    }

    pub(crate) fn replay_focuses(&self,scene:&PresentationScene,textures:&TextureScene)
        ->Option<[skate2_core::streaming::replay_focus::VisualFocus;2]> {
        use skate2_core::streaming::replay_focus::VisualFocus;
        let (pres,tex)=self.sources.as_ref()?;
        Some([
            VisualFocus {space:Arc::clone(&pres.space),map:Arc::clone(&pres.map),xml:pres.xml.clone(),radius:pres.radius,
                resident:scene.collections.iter().map(|c|c.id).collect(),texture:false},
            VisualFocus {space:Arc::clone(&tex.space),map:Arc::clone(&tex.map),xml:tex.xml.clone(),radius:tex.radius,
                resident:textures.collections.iter().map(|c|c.id).collect(),texture:true},
        ])
    }

    pub(crate) fn activate_proxy_world(
        &mut self,
        uploader: GpuTextureUploader,
        scene: &mut PresentationScene,
    ) -> RenderResult<ProxyTextures> {
        let order = self
            .proxy
            .lock()
            .map_err(|_| "PROXY world lock poisoned")?
            .collection_ids();
        let (textures, geometry) = ProxyTextures::load(
            Arc::clone(&self.mounts),
            self.catalog.clone(),
            uploader,
            &order,
        )?;
        let mut owner = self.proxy.lock().map_err(|_| "PROXY world lock poisoned")?;
        owner.activate(geometry)?;
        for collection in &scene.collections {
            owner.high_registered(collection.id)?;
        }
        scene.proxies = owner.registered_collections();
        self.proxy_revision = Some(owner.revision()?);
        eprintln!(
            "PROXY world activated: {} registered proxies beside {} high collections",
            scene.proxies.len(),
            scene.collections.len()
        );
        Ok(textures)
    }

    /// Transfer the existing CPU owners to one loader while the immutable GPU
    /// draw generation stays live. No scene cloning or concurrent second loader.
    /// Return prepared render state only when a complete replacement is available.
    pub(crate) fn update(
        &mut self,
        scene: &mut Option<PresentationScene>,
        textures: &mut Option<TextureScene>,
        position: [f32; 3],
        disabled: bool,
        xml_enabled: bool,
        hulls: &[HullPresentation],
    ) -> RenderResult<Option<PreparedRenderState>> {
        if let Some(pending) = &self.pending {
            match pending.try_recv() {
                Ok(result) => {
                    self.pending = None;
                    let (loaded_scene, loaded_textures, render_state, proxy_revision) = result?;
                    *scene = Some(loaded_scene);
                    *textures = Some(loaded_textures);
                    self.proxy_revision = proxy_revision;
                    return Ok(Some(render_state));
                }
                Err(mpsc::TryRecvError::Empty) => return Ok(None),
                Err(mpsc::TryRecvError::Disconnected) => {
                    return Err("city loader stopped without completion".into());
                }
            }
        }
        // A proxy registration change without a focus change (a mapping that
        // arrived after its high collection registered) still needs a new
        // draw list.
        let proxy_changed = match self.proxy_revision {
            Some(built) => {
                self.proxy
                    .lock()
                    .map_err(|_| "PROXY world lock poisoned")?
                    .revision()?
                    != built
            }
            None => false,
        };
        let focus_enabled = !disabled && !position.iter().any(|x| x.is_nan());
        // A hull registration change also needs a new draw list.
        let hulls_changed = scene.as_ref().is_some_and(|scene| {
            !scene.hulls.iter().map(|c| c.id).eq(hulls.iter().flat_map(|hull| hull.collections.iter().copied()))
        });
        if !focus_enabled && !proxy_changed && !hulls_changed {
            return Ok(None);
        }
        let current_scene = scene.as_mut().ok_or("city scene owner absent")?;
        let current_textures = textures.as_mut().ok_or("city texture owner absent")?;
        let mut moved = None;
        if focus_enabled {
            if self.sources.is_none() {
                self.sources = Some((
                    FocusSource::from_registered(
                        &self.catalog,
                        current_scene
                            .collections
                            .first()
                            .ok_or("city has no presentation seed")?
                            .id,
                        StreamKind::Presentation,
                        WORLD_FOCUS_RADIUS,
                    )?,
                    FocusSource::from_registered(
                        &self.catalog,
                        current_textures
                            .collections
                            .first()
                            .ok_or("city has no texture seed")?
                            .id,
                        StreamKind::Texture,
                        TEXTURE_FOCUS_RADIUS,
                    )?,
                ));
            }
            let (pres, tex) = self.sources.as_ref().expect("sources initialized above");
            let selected_pres = pres.select(position, xml_enabled);
            let selected_tex = tex.select(position, xml_enabled);
            let ids = |selected: &[FocusCollection]| {
                selected.iter().map(|c| c.id).collect::<BTreeSet<_>>()
            };
            let changed = ids(&selected_pres)
                != current_scene.collections.iter().map(|c| c.id).collect()
                || ids(&selected_tex)
                    != current_textures.collections.iter().map(|c| c.id).collect();
            if changed {
                moved = Some((
                    selected_pres,
                    selected_tex,
                    Arc::clone(&pres.map),
                    Arc::clone(&tex.map),
                ));
            } else {
                // Do not reorder index-based handles just because distances change.
                current_scene.stream_focus.move_to(position, false);
                current_scene.stream_focus.reconcile(&selected_pres);
                current_scene.focus = position;
                current_textures.stream_focus.move_to(position, false);
                current_textures.stream_focus.reconcile(&selected_tex);
                current_textures.focus = position;
                if !proxy_changed && !hulls_changed {
                    return Ok(None);
                }
            }
        }
        let mut scene_owner = scene.take().expect("scene checked above");
        let mut texture_owner = textures.take().expect("textures checked above");
        let mounts = Arc::clone(&self.mounts);
        let proxy = Arc::clone(&self.proxy);
        let proxy_active = self.proxy_revision.is_some();
        let hulls = hulls.to_vec();
        let (send, receive) = mpsc::sync_channel(1);
        thread::Builder::new()
            .name("city-resource-loader".into())
            .spawn(move || {
                let result = (|| -> RenderResult<_> {
                    let mut transition = None;
                    if let Some((selected_pres, selected_tex, pres_map, tex_map)) = moved {
                        transition = Some(crate::move_runtime_presentation_selection(
                            &mut scene_owner,
                            &mounts,
                            position,
                            &pres_map,
                            &selected_pres,
                        )?);
                        crate::move_runtime_texture_selection(
                            &mut texture_owner,
                            &mounts,
                            position,
                            &tex_map,
                            &selected_tex,
                        )?;
                        scene_owner.bind_resident_textures(&texture_owner);
                    }
                    if crate::hull_presentation::sync(&mut scene_owner, &mounts, &hulls)? {
                        crate::bind_runtime_material_attributes(&mounts, &mut scene_owner)?;
                        scene_owner.bind_resident_textures(&texture_owner);
                    }
                    let proxy_revision = register_proxies(
                        &proxy,
                        proxy_active,
                        transition.as_ref(),
                        &mut scene_owner,
                    )?;
                    let render_state =
                        crate::render_inspection::build_render_state(&scene_owner, &mounts)?;
                    Ok((scene_owner, texture_owner, render_state, proxy_revision))
                })()
                .map_err(|error| error.to_string());
                let _ = send.send(result);
            })?;
        self.pending = Some(receive);
        Ok(None)
    }
}
