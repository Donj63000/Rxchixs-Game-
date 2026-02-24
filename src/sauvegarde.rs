use super::*;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const SAVE_FILE_EXTENSION: &str = "ron";

#[derive(Clone, Serialize, Deserialize)]
struct SauvegardeDocument {
    #[serde(default = "default_save_schema_version")]
    schema_version: u32,
    save_name: String,
    saved_at_unix_s: i64,
    map: MapAsset,
}

fn default_save_schema_version() -> u32 {
    SAVE_SCHEMA_VERSION
}

#[derive(Clone, Debug)]
pub(crate) struct SauvegardeInfo {
    pub file_name: String,
    pub save_name: String,
    pub saved_at_unix_s: i64,
    pub saved_at_label: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ListeSauvegardes {
    pub slots: Vec<SauvegardeInfo>,
    pub warnings: Vec<String>,
}

pub(crate) fn now_unix_seconds() -> i64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_secs().min(i64::MAX as u64) as i64,
        Err(_) => 0,
    }
}

fn div_floor(a: i64, b: i64) -> i64 {
    let mut q = a / b;
    let r = a % b;
    if r != 0 && ((r > 0) != (b > 0)) {
        q -= 1;
    }
    q
}

fn rem_floor(a: i64, b: i64) -> i64 {
    a - div_floor(a, b) * b
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

fn timestamp_parts_utc(unix_s: i64) -> (i32, u32, u32, u32, u32, u32) {
    let days = div_floor(unix_s, 86_400);
    let sec_of_day = rem_floor(unix_s, 86_400);
    let (year, month, day) = civil_from_days(days);
    let hour = (sec_of_day / 3_600) as u32;
    let minute = ((sec_of_day % 3_600) / 60) as u32;
    let second = (sec_of_day % 60) as u32;
    (year, month, day, hour, minute, second)
}

pub(crate) fn format_horodate_utc(unix_s: i64) -> String {
    let (year, month, day, hour, minute, second) = timestamp_parts_utc(unix_s);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year, month, day, hour, minute, second
    )
}

fn format_horodate_for_file(unix_s: i64) -> String {
    let (year, month, day, hour, minute, second) = timestamp_parts_utc(unix_s);
    format!(
        "{:04}{:02}{:02}_{:02}{:02}{:02}",
        year, month, day, hour, minute, second
    )
}

pub(crate) fn proposer_nom_sauvegarde(unix_s: i64) -> String {
    let (year, month, day, hour, minute, second) = timestamp_parts_utc(unix_s);
    format!(
        "Sauvegarde {:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hour, minute, second
    )
}

fn sanitize_save_name_for_label(raw_name: &str) -> String {
    let trimmed = raw_name.trim();
    if trimmed.is_empty() {
        return "Sauvegarde".to_string();
    }
    let mut out = String::with_capacity(trimmed.len().min(64));
    for ch in trimmed.chars() {
        let keep = ch.is_ascii_alphanumeric() || matches!(ch, ' ' | '_' | '-' | '.');
        if keep {
            out.push(ch);
        }
        if out.len() >= 64 {
            break;
        }
    }
    let out = out.trim().to_string();
    if out.is_empty() {
        "Sauvegarde".to_string()
    } else {
        out
    }
}

fn sanitize_save_name_for_file(raw_name: &str) -> String {
    let label = sanitize_save_name_for_label(raw_name);
    let mut slug = String::with_capacity(label.len());
    for ch in label.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
        } else if matches!(ch, ' ' | '_' | '-') {
            slug.push('_');
        }
    }
    while slug.contains("__") {
        slug = slug.replace("__", "_");
    }
    let slug = slug.trim_matches('_').to_string();
    if slug.is_empty() {
        "sauvegarde".to_string()
    } else {
        slug.chars().take(40).collect()
    }
}

fn save_slot_from_doc(file_name: String, document: &SauvegardeDocument) -> SauvegardeInfo {
    SauvegardeInfo {
        file_name,
        save_name: sanitize_save_name_for_label(&document.save_name),
        saved_at_unix_s: document.saved_at_unix_s,
        saved_at_label: format_horodate_utc(document.saved_at_unix_s),
    }
}

fn validate_schema(schema_version: u32) -> Result<(), String> {
    if schema_version > SAVE_SCHEMA_VERSION {
        return Err(format!(
            "schema de sauvegarde futur non supporte ({} > {})",
            schema_version, SAVE_SCHEMA_VERSION
        ));
    }
    Ok(())
}

fn save_file_path(dir: &Path, file_name: &str) -> Result<PathBuf, String> {
    let name = file_name.trim();
    if name.is_empty() {
        return Err("nom de fichier de sauvegarde vide".to_string());
    }
    if name.contains(['/', '\\']) {
        return Err("nom de fichier de sauvegarde invalide".to_string());
    }
    let path = dir.join(name);
    if path.extension().and_then(|ext| ext.to_str()) != Some(SAVE_FILE_EXTENSION) {
        return Err(format!(
            "extension de sauvegarde invalide, attendu .{}",
            SAVE_FILE_EXTENSION
        ));
    }
    Ok(path)
}

fn read_save_doc(path: &Path) -> Result<SauvegardeDocument, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("lecture impossible ({:?}): {}", path.file_name(), err))?;
    let mut document: SauvegardeDocument = ron_from_str(&raw)
        .map_err(|err| format!("format RON invalide ({:?}): {}", path.file_name(), err))?;
    validate_schema(document.schema_version)?;
    document.save_name = sanitize_save_name_for_label(&document.save_name);
    sanitize_map_asset(&mut document.map);
    Ok(document)
}

fn unique_file_name(dir: &Path, slug: &str, unix_s: i64) -> String {
    let stamp = format_horodate_for_file(unix_s);
    let base = format!("{}_{}", stamp, slug);
    let mut candidate = format!("{}.{}", base, SAVE_FILE_EXTENSION);
    if !dir.join(&candidate).exists() {
        return candidate;
    }
    let mut suffix = 2u32;
    loop {
        candidate = format!("{}_{}.{}", base, suffix, SAVE_FILE_EXTENSION);
        if !dir.join(&candidate).exists() {
            return candidate;
        }
        suffix += 1;
    }
}

fn lister_sauvegardes_dans(dir: &Path) -> Result<ListeSauvegardes, String> {
    fs::create_dir_all(dir)
        .map_err(|err| format!("impossible de creer le dossier de sauvegardes: {err}"))?;

    let mut slots = Vec::new();
    let mut warnings = Vec::new();
    let entries =
        fs::read_dir(dir).map_err(|err| format!("impossible de lister les sauvegardes: {err}"))?;

    for entry_res in entries {
        let entry = match entry_res {
            Ok(entry) => entry,
            Err(err) => {
                warnings.push(format!("entree de sauvegarde ignoree: {err}"));
                continue;
            }
        };
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some(SAVE_FILE_EXTENSION) {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        match read_save_doc(&path) {
            Ok(document) => slots.push(save_slot_from_doc(file_name, &document)),
            Err(err) => warnings.push(format!("{file_name}: {err}")),
        }
    }

    slots.sort_by(|a, b| {
        b.saved_at_unix_s
            .cmp(&a.saved_at_unix_s)
            .then_with(|| a.file_name.cmp(&b.file_name))
    });

    Ok(ListeSauvegardes { slots, warnings })
}

pub(crate) fn lister_sauvegardes() -> Result<ListeSauvegardes, String> {
    lister_sauvegardes_dans(Path::new(SAVE_DIR_PATH))
}

fn enregistrer_sauvegarde_dans(
    dir: &Path,
    map: &MapAsset,
    save_name: &str,
    unix_s: i64,
) -> Result<SauvegardeInfo, String> {
    fs::create_dir_all(dir)
        .map_err(|err| format!("impossible de creer le dossier de sauvegardes: {err}"))?;

    let clean_name = sanitize_save_name_for_label(save_name);
    let slug = sanitize_save_name_for_file(&clean_name);
    let file_name = unique_file_name(dir, &slug, unix_s);
    let path = dir.join(&file_name);
    let mut map_copy = map.clone();
    sanitize_map_asset(&mut map_copy);

    let document = SauvegardeDocument {
        schema_version: SAVE_SCHEMA_VERSION,
        save_name: clean_name,
        saved_at_unix_s: unix_s,
        map: map_copy,
    };
    let pretty = PrettyConfig::new()
        .depth_limit(5)
        .enumerate_arrays(true)
        .separate_tuple_members(true);
    let payload = ron_to_string_pretty(&document, pretty)
        .map_err(|err| format!("echec serialisation sauvegarde: {err}"))?;
    fs::write(&path, payload).map_err(|err| format!("echec ecriture sauvegarde: {err}"))?;
    Ok(save_slot_from_doc(file_name, &document))
}

pub(crate) fn enregistrer_sauvegarde(
    map: &MapAsset,
    save_name: &str,
) -> Result<SauvegardeInfo, String> {
    let unix_s = now_unix_seconds();
    enregistrer_sauvegarde_dans(Path::new(SAVE_DIR_PATH), map, save_name, unix_s)
}

fn charger_sauvegarde_depuis(dir: &Path, file_name: &str) -> Result<MapAsset, String> {
    let path = save_file_path(dir, file_name)?;
    let document = read_save_doc(&path)?;
    Ok(document.map)
}

pub(crate) fn charger_sauvegarde(file_name: &str) -> Result<MapAsset, String> {
    charger_sauvegarde_depuis(Path::new(SAVE_DIR_PATH), file_name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process;

    fn test_save_dir(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        dir.push(format!("rxchixs_save_tests_{}_{}", name, process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("test save dir should be creatable");
        dir
    }

    #[test]
    fn horodate_utc_is_stable() {
        assert_eq!(format_horodate_utc(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(
            format_horodate_utc(1_704_067_200),
            "2024-01-01 00:00:00 UTC"
        );
    }

    #[test]
    fn sanitize_name_for_file_keeps_safe_slug() {
        assert_eq!(sanitize_save_name_for_file("  Ma Save #1 !  "), "ma_save_1");
        assert_eq!(sanitize_save_name_for_file("%%%%"), "sauvegarde");
    }

    #[test]
    fn save_list_and_load_roundtrip() {
        let dir = test_save_dir("roundtrip");
        let map = MapAsset::new_default();

        let slot = enregistrer_sauvegarde_dans(&dir, &map, "Test Save", 1_700_000_000)
            .expect("save should succeed");
        let listing = lister_sauvegardes_dans(&dir).expect("listing should succeed");
        assert_eq!(listing.warnings.len(), 0);
        assert_eq!(listing.slots.len(), 1);
        assert_eq!(listing.slots[0].save_name, "Test Save");
        assert_eq!(listing.slots[0].file_name, slot.file_name);

        let loaded = charger_sauvegarde_depuis(&dir, &slot.file_name).expect("load should succeed");
        assert_eq!(loaded.world.w, map.world.w);
        assert_eq!(loaded.world.h, map.world.h);
        assert_eq!(loaded.player_spawn, map.player_spawn);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_reports_invalid_files_as_warnings() {
        let dir = test_save_dir("invalid");
        let bad_path = dir.join("bad_save.ron");
        fs::write(&bad_path, "this is not ron").expect("invalid file should be written");

        let listing = lister_sauvegardes_dans(&dir).expect("listing should not fail");
        assert_eq!(listing.slots.len(), 0);
        assert_eq!(listing.warnings.len(), 1);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_rejects_future_schema() {
        let dir = test_save_dir("future_schema");
        let map = MapAsset::new_default();
        let bad_doc = SauvegardeDocument {
            schema_version: SAVE_SCHEMA_VERSION + 1,
            save_name: "future".to_string(),
            saved_at_unix_s: 1_700_000_100,
            map,
        };
        let payload = ron_to_string_pretty(&bad_doc, PrettyConfig::new())
            .expect("future doc should serialize");
        let path = dir.join("future_schema.ron");
        fs::write(&path, payload).expect("future schema file should be written");

        let err = match charger_sauvegarde_depuis(&dir, "future_schema.ron") {
            Ok(_) => panic!("future schema should be rejected"),
            Err(err) => err,
        };
        assert!(err.contains("schema"));

        let _ = fs::remove_dir_all(&dir);
    }
}
