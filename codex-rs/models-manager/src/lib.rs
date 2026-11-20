pub mod cache;
mod catalog;
pub mod collaboration_mode_presets;
pub(crate) mod config;
pub mod manager;
pub mod model_info;
pub mod model_presets;
mod originator;
pub mod test_support;

pub use catalog::CatalogModel;
pub use catalog::ModelOrigin;
pub use codex_protocol::auth::AuthMode;
pub use config::ModelsManagerConfig;
pub use originator::ModelCatalogOriginator;

/// Load the bundled model catalog shipped with `codex-models-manager`.
pub fn bundled_models_response()
-> std::result::Result<codex_protocol::openai_models::ModelsResponse, serde_json::Error> {
    serde_json::from_str(include_str!("../models.json"))
}

/// Model aliases the Zaino gateway serves, in picker order.
///
/// The gateway rejects every upstream slug with "Unknown model", so these are
/// the only models that can actually run against it.
pub const ZAINO_MODEL_SLUGS: [&str; 4] = ["smart", "fast", "frontier", "fable"];

/// Narrow the bundled catalog to the Zaino aliases.
///
/// `models.json` is shared with the Bedrock, Vertex and first-party providers,
/// whose catalogs are keyed on the upstream slugs, so the aliases cannot simply
/// replace the other entries there. It already carries all four Zaino models
/// with full Zaino prompts and display names; they only sit behind
/// `visibility: none` because the first-party picker hides them. Promoting them
/// here means the picker offers exactly what the gateway accepts, and the
/// catalog cannot drift from the shared file.
pub fn zaino_models_response()
-> std::result::Result<codex_protocol::openai_models::ModelsResponse, serde_json::Error> {
    use codex_protocol::openai_models::ModelVisibility;

    let mut catalog = bundled_models_response()?;
    catalog
        .models
        .retain(|model| ZAINO_MODEL_SLUGS.contains(&model.slug.as_str()));
    catalog
        .models
        .sort_by_key(|model| zaino_model_rank(&model.slug));
    for (index, model) in catalog.models.iter_mut().enumerate() {
        model.visibility = ModelVisibility::List;
        model.priority = index as i32 + 1;
    }
    Ok(catalog)
}

/// Position of a slug in [`ZAINO_MODEL_SLUGS`], which also fixes picker order.
fn zaino_model_rank(slug: &str) -> usize {
    ZAINO_MODEL_SLUGS
        .iter()
        .position(|candidate| candidate == &slug)
        .unwrap_or(ZAINO_MODEL_SLUGS.len())
}

/// Convert the client version string to a whole version string (e.g. "1.2.3-alpha.4" -> "1.2.3").
pub fn client_version_to_whole() -> String {
    format!(
        "{}.{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The Zaino gateway rejects every upstream slug, so offering one would give
    /// the user a model that cannot run. Guard against the OpenAI catalog
    /// leaking back in.
    #[test]
    fn zaino_catalog_contains_only_gateway_aliases() {
        let catalog = zaino_models_response().expect("zaino catalog should parse");
        let slugs: Vec<&str> = catalog.models.iter().map(|m| m.slug.as_str()).collect();
        assert_eq!(slugs, ZAINO_MODEL_SLUGS);
    }

    /// Every alias must be visible as well as listed, or the picker hides the
    /// very models the gateway accepts.
    #[test]
    fn zaino_catalog_marks_every_alias_visible() {
        use codex_protocol::openai_models::ModelVisibility;

        let catalog = zaino_models_response().expect("zaino catalog should parse");
        for model in &catalog.models {
            assert_eq!(
                model.visibility,
                ModelVisibility::List,
                "{} should be picker-visible",
                model.slug
            );
            assert!(
                model.supported_in_api,
                "{} should be API-supported",
                model.slug
            );
        }
    }

    /// The catalog is narrowed from the shared file rather than duplicated, so
    /// the aliases keep the Zaino prompts they already ship with.
    #[test]
    fn zaino_catalog_keeps_the_zaino_prompt() {
        let catalog = zaino_models_response().expect("zaino catalog should parse");
        for model in &catalog.models {
            let instructions = model
                .model_messages
                .as_ref()
                .and_then(|messages| messages.instructions_template.as_deref())
                .unwrap_or_default();
            assert!(
                instructions.starts_with("You are Zaino,"),
                "{} should carry the Zaino instructions",
                model.slug
            );
        }
    }

    /// `smart` leads the list and is API-supported, which is what makes it the
    /// default preset. Reordering `ZAINO_MODEL_SLUGS` would silently change the
    /// default.
    #[test]
    fn zaino_default_model_is_smart() {
        use codex_protocol::openai_models::ModelPreset;

        let catalog = zaino_models_response().expect("zaino catalog should parse");
        let mut presets: Vec<ModelPreset> =
            catalog.models.into_iter().map(ModelPreset::from).collect();
        ModelPreset::mark_default_by_picker_visibility(&mut presets);
        let default = presets
            .iter()
            .find(|preset| preset.is_default)
            .expect("one preset should be the default");
        assert_eq!(default.model, "smart");
    }
}
