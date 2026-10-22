//! Explicit offline census: production decoders and Vault bindings, one
//! collection at a time. No window, shader substitution or simulation changes.
use crate::gpu_renderer::{EnvironmentEffect, RenderResult};
use skate2_core::presentation::{PresentationMaterialEffect, PresentationMaterialValue};
use skate2_formats::stream::{CollectionMap, FocusCollection};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Entry {
    materials: usize,
    channels: BTreeMap<String, usize>,
    effect: Option<PresentationMaterialEffect>,
    sample_collection: String,
}

pub(crate) fn run() -> RenderResult<()> {
    let root = std::env::var_os("SKATE2_GAME_ROOT").ok_or("set SKATE2_GAME_ROOT")?;
    let mounts = crate::mount_reference_archives(std::path::Path::new(&root))?;
    let map =
        CollectionMap::parse(&mounts.read_path(r"data\content\world\stream\BAM\BAM_Pres.xsm")?)?;
    let mut entries = BTreeMap::<String, Entry>::new();
    let mut lightmap_mips = BTreeMap::<u32, usize>::new();
    let mut lightmap_guids = BTreeSet::new();
    for (index, row) in map.collections.iter().enumerate() {
        let focus = FocusCollection {
            index,
            id: row.id,
            position: [0.; 3],
            distance_xz: 0.,
            stream_order: 0,
        };
        let collection = crate::load_presentation_collection(&mounts, &map, &focus)?;
        for arena in &collection.arenas {
            for material in &arena.materials {
                let name = material
                    .effect
                    .as_ref()
                    .map_or("<no effect>", |e| &e.serialized_name);
                let entry = entries.entry(name.to_owned()).or_default();
                entry.materials += 1;
                entry.effect = material.effect.clone();
                if entry.sample_collection.is_empty() {
                    entry.sample_collection = row.filename.clone();
                }
                for channel in material.channels.iter().skip(2) {
                    *entry
                        .channels
                        .entry(channel.shader_input.clone())
                        .or_default() += 1;
                    if channel.shader_input == "lightmap" {
                        if let PresentationMaterialValue::Raster { guid, .. } = channel.value {
                            lightmap_guids.insert(guid);
                            if let Some(texture) = arena.textures.iter().find(|t| t.guid == guid) {
                                *lightmap_mips.entry(texture.mip_levels).or_default() += 1;
                            }
                        }
                    }
                }
            }
        }
        std::eprintln!(
            "material census {}/{}: {}",
            index + 1,
            map.collections.len(),
            row.filename
        );
    }
    let keys = entries
        .values()
        .filter_map(|e| e.effect.as_ref())
        .filter_map(|e| e.attribute_class_id.zip(e.material_variant_id))
        .collect::<BTreeSet<_>>();
    let bindings = crate::load_material_attribute_bindings(&mounts, &keys)?;
    std::println!("material\tmanifest\tported\tcount\tsample_collection\tchannels");
    for (name, entry) in entries {
        let manifest = entry
            .effect
            .as_ref()
            .and_then(|e| e.attribute_class_id.zip(e.material_variant_id))
            .and_then(|key| bindings.get(&key))
            .map_or("<native default fallback>", |b| b.manifest_name.as_str());
        std::println!(
            "{}\t{}\t{}\t{}\t{}\t{:?}",
            name,
            manifest,
            EnvironmentEffect::from_manifest(manifest).is_some(),
            entry.materials,
            entry.sample_collection,
            entry.channels
        );
    }
    let sampled =
        crate::load_runtime_texture_scene(&mounts, skate2_core::player::selected_spawn_location())?;
    let mut sampled_lightmaps = BTreeMap::<(u32, u32, u32), usize>::new();
    for texture in sampled
        .collections
        .iter()
        .flat_map(|c| &c.arenas)
        .flat_map(|a| &a.textures)
    {
        if lightmap_guids.contains(&texture.guid) {
            *sampled_lightmaps
                .entry((texture.width, texture.height, texture.mip_levels))
                .or_default() += 1;
        }
    }
    std::println!("arena-local lightmap mip-level histogram: {lightmap_mips:?}");
    std::println!("spawn-region lightmap (width,height,mips) histogram: {sampled_lightmaps:?}");
    Ok(())
}
