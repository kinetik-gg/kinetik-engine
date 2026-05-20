use crate::{
    AssetGuid, AssetPath, ImportCacheRecord, ImportCacheSchemaVersion, ImportSettingsHash,
    ResourceError, ResourceResult, SourceContentHash,
};

/// Kinetik texture importer identifier.
pub const TEXTURE_IMPORTER_ID: &str = "kinetik.texture";

/// Kinetik glTF/GLB mesh importer identifier.
pub const GLTF_IMPORTER_ID: &str = "kinetik.gltf";

/// Kinetik material importer identifier.
pub const MATERIAL_IMPORTER_ID: &str = "kinetik.material";

/// Initial importer contract version.
pub const IMPORTER_VERSION: &str = "1.0.0";

/// Initial generated import cache schema version.
pub const IMPORT_CACHE_SCHEMA_VERSION: ImportCacheSchemaVersion = ImportCacheSchemaVersion::new(1);

/// Asset kind supported by the current import foundation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetImportKind {
    /// PNG/JPEG texture source.
    Texture,
    /// glTF or GLB mesh source.
    GltfMesh,
    /// Kinetik material source.
    Material,
}

impl AssetImportKind {
    /// Returns the importer ID for this asset kind.
    #[must_use]
    pub const fn importer_id(self) -> &'static str {
        match self {
            Self::Texture => TEXTURE_IMPORTER_ID,
            Self::GltfMesh => GLTF_IMPORTER_ID,
            Self::Material => MATERIAL_IMPORTER_ID,
        }
    }

    /// Returns the expected extension description for diagnostics.
    #[must_use]
    pub const fn expected_extensions(self) -> &'static str {
        match self {
            Self::Texture => ".png, .jpg, or .jpeg",
            Self::GltfMesh => ".gltf or .glb",
            Self::Material => ".knmat",
        }
    }

    fn accepts_path(self, path: &AssetPath) -> bool {
        let extension = asset_path_extension(path);
        match self {
            Self::Texture => extension_is_one_of(extension, &["png", "jpg", "jpeg"]),
            Self::GltfMesh => extension_is_one_of(extension, &["gltf", "glb"]),
            Self::Material => extension_is_one_of(extension, &["knmat"]),
        }
    }
}

/// Deterministic import request validated before parser-specific import work.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AssetImportRequest {
    guid: AssetGuid,
    path: AssetPath,
    kind: AssetImportKind,
    source_content_hash: SourceContentHash,
    settings_hash: ImportSettingsHash,
}

impl AssetImportRequest {
    /// Creates a validated import request.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::InvalidAssetKind`] when the path extension does
    /// not match the requested import kind.
    pub fn new(
        guid: AssetGuid,
        path: AssetPath,
        kind: AssetImportKind,
        source_content_hash: SourceContentHash,
        settings_hash: ImportSettingsHash,
    ) -> ResourceResult<Self> {
        validate_asset_kind(path.clone(), kind)?;
        Ok(Self {
            guid,
            path,
            kind,
            source_content_hash,
            settings_hash,
        })
    }

    /// Returns stable asset identity.
    #[must_use]
    pub const fn guid(&self) -> AssetGuid {
        self.guid
    }

    /// Returns validated source asset path.
    #[must_use]
    pub const fn path(&self) -> &AssetPath {
        &self.path
    }

    /// Returns import kind.
    #[must_use]
    pub const fn kind(&self) -> AssetImportKind {
        self.kind
    }

    /// Returns source content hash.
    #[must_use]
    pub const fn source_content_hash(&self) -> &SourceContentHash {
        &self.source_content_hash
    }

    /// Returns import settings hash.
    #[must_use]
    pub const fn settings_hash(&self) -> &ImportSettingsHash {
        &self.settings_hash
    }

    /// Creates import cache metadata for this request.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] if importer metadata validation fails.
    pub fn to_cache_record(&self) -> ResourceResult<ImportCacheRecord> {
        ImportCacheRecord::new(
            self.guid,
            self.source_content_hash.clone(),
            self.kind.importer_id(),
            IMPORTER_VERSION,
            self.settings_hash.clone(),
            IMPORT_CACHE_SCHEMA_VERSION,
        )
    }
}

/// Deterministic artifact metadata produced by an importer smoke.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImportArtifactRecord {
    guid: AssetGuid,
    source_path: AssetPath,
    kind: AssetImportKind,
    cache_record: ImportCacheRecord,
    artifact_path: String,
}

impl ImportArtifactRecord {
    /// Creates deterministic artifact metadata from a validated import request.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError`] if cache metadata validation fails.
    pub fn from_request(request: &AssetImportRequest) -> ResourceResult<Self> {
        let cache_record = request.to_cache_record()?;
        Ok(Self {
            guid: request.guid(),
            source_path: request.path().clone(),
            kind: request.kind(),
            artifact_path: artifact_path_for(request),
            cache_record,
        })
    }

    /// Returns stable asset identity.
    #[must_use]
    pub const fn guid(&self) -> AssetGuid {
        self.guid
    }

    /// Returns source asset path.
    #[must_use]
    pub const fn source_path(&self) -> &AssetPath {
        &self.source_path
    }

    /// Returns import kind.
    #[must_use]
    pub const fn kind(&self) -> AssetImportKind {
        self.kind
    }

    /// Returns import cache metadata.
    #[must_use]
    pub const fn cache_record(&self) -> &ImportCacheRecord {
        &self.cache_record
    }

    /// Returns deterministic generated artifact path.
    #[must_use]
    pub fn artifact_path(&self) -> &str {
        &self.artifact_path
    }
}

/// Validates that an asset path matches an expected import kind.
///
/// # Errors
///
/// Returns [`ResourceError::InvalidAssetKind`] when the path extension does not
/// match the expected kind.
pub fn validate_asset_kind(path: AssetPath, kind: AssetImportKind) -> ResourceResult<AssetPath> {
    if kind.accepts_path(&path) {
        Ok(path)
    } else {
        Err(ResourceError::InvalidAssetKind {
            path,
            expected: kind.expected_extensions(),
        })
    }
}

fn artifact_path_for(request: &AssetImportRequest) -> String {
    let folder = match request.kind() {
        AssetImportKind::Texture => "textures",
        AssetImportKind::GltfMesh => "meshes",
        AssetImportKind::Material => "materials",
    };
    format!(
        ".kinetik/import/{folder}/{}-{}.kasset",
        request.guid().raw(),
        request.source_content_hash().as_str()
    )
}

fn asset_path_extension(path: &AssetPath) -> Option<&str> {
    path.as_str()
        .rsplit_once('.')
        .map(|(_, extension)| extension)
        .filter(|extension| !extension.is_empty())
}

fn extension_is_one_of(extension: Option<&str>, allowed: &[&str]) -> bool {
    let Some(extension) = extension else {
        return false;
    };
    allowed
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
}
