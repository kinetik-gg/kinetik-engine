use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use kinetik_command::CommandHistory;
use kinetik_project::{ProjectDocumentRefs, ProjectModel, ProjectSettingsDocument};
use kinetik_resource::AssetManifest;
use kinetik_scene::{InstanceClassRegistry, Scene, SceneDocument};

use crate::EditorSession;

/// Error returned by editor project save/reload operations.
#[derive(Debug)]
pub enum EditorPersistenceError {
    /// No project/scene is open in the editor session.
    NoActiveProject,
    /// File I/O failed.
    Io {
        /// Affected path.
        path: PathBuf,
        /// Underlying I/O error.
        source: io::Error,
    },
    /// Project settings parsing or writing failed.
    Project(kinetik_project::ProjectError),
    /// Asset manifest parsing or writing failed.
    Resource(kinetik_resource::ResourceError),
    /// Scene parsing, writing, or validation failed.
    Scene(kinetik_scene::SceneError),
    /// Saved snapshots did not match the open session after writing.
    SnapshotMismatch,
}

impl fmt::Display for EditorPersistenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoActiveProject => formatter.write_str("no project is open"),
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Project(error) => write!(formatter, "{error}"),
            Self::Resource(error) => write!(formatter, "{error}"),
            Self::Scene(error) => write!(formatter, "{error}"),
            Self::SnapshotMismatch => formatter.write_str("saved project snapshot did not match"),
        }
    }
}

impl std::error::Error for EditorPersistenceError {}

impl EditorSession {
    /// Saves active project settings, scene, and asset manifest files.
    ///
    /// # Errors
    ///
    /// Returns [`EditorPersistenceError`] when no project is open, file I/O
    /// fails, serialization fails, or saved snapshots do not match the session.
    pub fn save_project_to(&mut self, root: &Path) -> Result<(), EditorPersistenceError> {
        let project = self
            .project
            .as_ref()
            .ok_or(EditorPersistenceError::NoActiveProject)?;
        let scene = self
            .active_scene
            .as_ref()
            .ok_or(EditorPersistenceError::NoActiveProject)?;
        let settings_text = project
            .settings()
            .to_toml_string()
            .map_err(EditorPersistenceError::Project)?;
        let scene_document = scene.to_document().map_err(EditorPersistenceError::Scene)?;
        let scene_text = scene_document
            .to_ron_string()
            .map_err(EditorPersistenceError::Scene)?;
        let manifest_text = self
            .asset_manifest
            .to_toml_string()
            .map_err(EditorPersistenceError::Resource)?;

        write_text(root.join("Kinetik.toml"), &settings_text)?;
        write_text(root.join(project.documents().active_scene()), &scene_text)?;
        write_text(
            root.join(project.documents().assets_manifest()),
            &manifest_text,
        )?;

        let saved_settings = ProjectSettingsDocument::from_toml_str(&settings_text)
            .map_err(EditorPersistenceError::Project)?;
        let saved_scene_document =
            SceneDocument::from_ron_str(&scene_text).map_err(EditorPersistenceError::Scene)?;
        let saved_manifest = AssetManifest::from_toml_str(&manifest_text)
            .map_err(EditorPersistenceError::Resource)?;
        if &saved_settings != project.settings()
            || saved_scene_document != scene_document
            || saved_manifest != self.asset_manifest
        {
            return Err(EditorPersistenceError::SnapshotMismatch);
        }

        self.command_history = CommandHistory::new();
        Ok(())
    }

    /// Reloads project settings, active scene, and asset manifest files from disk.
    ///
    /// # Errors
    ///
    /// Returns [`EditorPersistenceError`] when file I/O, parsing, or scene
    /// validation fails.
    pub fn reload_project_from(
        &mut self,
        root: &Path,
        class_registry: InstanceClassRegistry,
    ) -> Result<(), EditorPersistenceError> {
        let settings_text = read_text(root.join("Kinetik.toml"))?;
        let settings = ProjectSettingsDocument::from_toml_str(&settings_text)
            .map_err(EditorPersistenceError::Project)?;
        let documents = ProjectDocumentRefs::default();
        let scene_text = read_text(root.join(documents.active_scene()))?;
        let scene_document =
            SceneDocument::from_ron_str(&scene_text).map_err(EditorPersistenceError::Scene)?;
        let scene = Scene::from_document(class_registry, scene_document)
            .map_err(EditorPersistenceError::Scene)?;
        let manifest_text = read_text(root.join(documents.assets_manifest()))?;
        let asset_manifest = AssetManifest::from_toml_str(&manifest_text)
            .map_err(EditorPersistenceError::Resource)?;
        self.open_project_with_assets(
            ProjectModel::new(settings, documents),
            scene,
            asset_manifest,
        );
        Ok(())
    }
}

fn read_text(path: PathBuf) -> Result<String, EditorPersistenceError> {
    fs::read_to_string(&path).map_err(|source| EditorPersistenceError::Io { path, source })
}

fn write_text(path: PathBuf, contents: &str) -> Result<(), EditorPersistenceError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| EditorPersistenceError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    fs::write(&path, contents).map_err(|source| EditorPersistenceError::Io { path, source })
}
