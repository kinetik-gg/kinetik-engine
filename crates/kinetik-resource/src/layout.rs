use std::collections::BTreeSet;

use crate::{ResourceError, ResourceResult};

/// Logical purpose of a canonical project workspace path.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProjectPathKind {
    /// Project settings file.
    ProjectSettings,
    /// Scene source directory.
    ScenesDirectory,
    /// Default scene source file.
    MainScene,
    /// Prefab source directory.
    PrefabsDirectory,
    /// Script source directory.
    ScriptsDirectory,
    /// Source asset directory.
    AssetsDirectory,
    /// Stable metadata directory.
    ProjectMetadataDirectory,
    /// Asset manifest file.
    AssetsManifest,
    /// Instance manifest file.
    InstancesManifest,
    /// Generated Kinetik directory.
    KinetikDirectory,
    /// Generated cache directory.
    CacheDirectory,
    /// Generated import-output directory.
    ImportDirectory,
    /// Generated build-output directory.
    BuildDirectory,
}

/// Whether a project layout path is committed source or disposable generated output.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ProjectPathDomain {
    /// Source-controlled project path.
    Source,
    /// Generated disposable path under `.kinetik/`.
    Generated,
}

/// Canonical workspace-relative project path.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProjectLayoutPath {
    /// Logical path kind.
    pub kind: ProjectPathKind,
    /// Workspace-relative path using `/` separators.
    pub path: &'static str,
    /// Source or generated path domain.
    pub domain: ProjectPathDomain,
}

impl ProjectLayoutPath {
    /// Returns whether this path is required for source project validation.
    #[must_use]
    pub const fn is_required_source(self) -> bool {
        matches!(self.domain, ProjectPathDomain::Source)
    }
}

/// In-memory model of the default Kinetik project workspace layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectLayout {
    paths: Vec<ProjectLayoutPath>,
}

impl ProjectLayout {
    /// Creates the default Kinetik project layout model.
    #[must_use]
    pub fn new() -> Self {
        Self {
            paths: DEFAULT_PROJECT_LAYOUT_PATHS.to_vec(),
        }
    }

    /// Returns all scaffold paths in deterministic order.
    #[must_use]
    pub fn scaffold_paths(&self) -> &[ProjectLayoutPath] {
        &self.paths
    }

    /// Returns source-controlled paths required for project validation.
    #[must_use]
    pub fn required_source_paths(&self) -> Vec<ProjectLayoutPath> {
        self.paths
            .iter()
            .copied()
            .filter(|path| path.is_required_source())
            .collect()
    }

    /// Returns a canonical path by logical kind.
    #[must_use]
    pub fn path(&self, kind: ProjectPathKind) -> Option<&'static str> {
        self.paths
            .iter()
            .find(|path| path.kind == kind)
            .map(|path| path.path)
    }

    /// Validates that a caller-provided path set contains every required source path.
    ///
    /// Generated `.kinetik/` paths are intentionally excluded because they are
    /// disposable and can be rebuilt.
    ///
    /// # Errors
    ///
    /// Returns [`ResourceError::MissingProjectPaths`] with all missing paths in
    /// deterministic layout order.
    pub fn validate_required_paths<I, P>(&self, paths: I) -> ResourceResult<()>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<str>,
    {
        let present_paths: BTreeSet<String> = paths
            .into_iter()
            .map(|path| normalize_project_path(path.as_ref()))
            .collect();
        let missing_paths: Vec<String> = self
            .required_source_paths()
            .into_iter()
            .map(|path| path.path)
            .filter(|path| !present_paths.contains(*path))
            .map(str::to_owned)
            .collect();

        if missing_paths.is_empty() {
            Ok(())
        } else {
            Err(ResourceError::MissingProjectPaths {
                paths: missing_paths,
            })
        }
    }
}

impl Default for ProjectLayout {
    fn default() -> Self {
        Self::new()
    }
}

const DEFAULT_PROJECT_LAYOUT_PATHS: [ProjectLayoutPath; 13] = [
    ProjectLayoutPath {
        kind: ProjectPathKind::ProjectSettings,
        path: "Kinetik.toml",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::ScenesDirectory,
        path: "scenes",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::MainScene,
        path: "scenes/main.ktscene",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::PrefabsDirectory,
        path: "prefabs",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::ScriptsDirectory,
        path: "scripts",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::AssetsDirectory,
        path: "assets",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::ProjectMetadataDirectory,
        path: "project",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::AssetsManifest,
        path: "project/assets.ktmanifest",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::InstancesManifest,
        path: "project/instances.ktmanifest",
        domain: ProjectPathDomain::Source,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::KinetikDirectory,
        path: ".kinetik",
        domain: ProjectPathDomain::Generated,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::CacheDirectory,
        path: ".kinetik/cache",
        domain: ProjectPathDomain::Generated,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::ImportDirectory,
        path: ".kinetik/import",
        domain: ProjectPathDomain::Generated,
    },
    ProjectLayoutPath {
        kind: ProjectPathKind::BuildDirectory,
        path: ".kinetik/build",
        domain: ProjectPathDomain::Generated,
    },
];

fn normalize_project_path(path: &str) -> String {
    path.trim_end_matches('/').replace('\\', "/")
}
