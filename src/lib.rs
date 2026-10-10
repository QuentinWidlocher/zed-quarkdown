use std::collections::HashMap;
use std::fs;

use zed_extension_api::{
    self as zed, Architecture, LanguageServerId, Os, Result, current_platform,
};

const NAME: &str = "quarkdown";
const REPO: &str = "iamgio/quarkdown";

const LANGUAGE_SERVER_ARG: &str = "language-server";

const BIN_DIRECTORY: &str = "quarkdown/bin/quarkdown";
const BIN_DIRECTORY_WINDOWS: &str = "quarkdown/bin/quarkdown.bat";

const ASSET_LINUX: &str = "quarkdown-linux-x64.zip";
const ASSET_WINDOWS: &str = "quarkdown-windows-x64.zip";
const ASSET_MACOS_SILICON: &str = "quarkdown-macos-aarch64.zip";
const ASSET_MACOS_INTEL: &str = "quarkdown-macos-x64.zip";

struct QuarkdownExtension {
    cached_binary_path: Option<String>,
}

impl QuarkdownExtension {
    fn is_windows() -> bool {
        matches!(current_platform().0, Os::Windows)
    }

    /// Path of the launcher inside the extracted release archive.
    /// Windows ships a batch script, other platforms a shell script.
    fn binary_path_in_archive() -> &'static str {
        if Self::is_windows() {
            BIN_DIRECTORY_WINDOWS
        } else {
            BIN_DIRECTORY
        }
    }

    /// Builds a Zed command, routing `.bat`/`.cmd` scripts through `cmd /c`
    /// so they can actually be spawned on Windows.
    fn build_command(command: String, mut args: Vec<String>, env: zed::EnvVars) -> zed::Command {
        let is_batch = command.ends_with(".bat") || command.ends_with(".cmd");
        if Self::is_windows() && is_batch {
            args.insert(0, command);
            args.insert(0, "/c".to_string());
            zed::Command {
                command: "cmd".to_string(),
                args,
                env,
            }
        } else {
            zed::Command {
                command,
                args,
                env,
            }
        }
    }

    fn user_binary_settings(worktree: &zed::Worktree) -> Option<zed::settings::CommandSettings> {
        zed::settings::LspSettings::for_worktree(NAME, worktree)
            .ok()
            .and_then(|settings| settings.binary)
    }

    fn env_vars_from(map: HashMap<String, String>) -> zed::EnvVars {
        map.into_iter().collect()
    }

    fn download_and_cache_binary(
        &mut self,
        language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<String> {
        if let Some(path) = &self.cached_binary_path
            && fs::metadata(path).is_ok_and(|stat| stat.is_file())
        {
            return Ok(path.clone());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let release = zed::latest_github_release(
            REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;

        let download_type = zed::DownloadedFileType::Zip;

        let asset_name = match current_platform() {
            (Os::Linux, _) => ASSET_LINUX,
            (Os::Windows, _) => ASSET_WINDOWS,
            (Os::Mac, Architecture::Aarch64) => ASSET_MACOS_SILICON,
            (Os::Mac, _) => ASSET_MACOS_INTEL,
        };

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| format!("no asset found matching {asset_name:?}"))?;

        let version_dir = format!("{NAME}-{}", release.version);
        let binary_path = format!("{version_dir}/{}", Self::binary_path_in_archive());

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(&asset.download_url, &version_dir, download_type)
                .map_err(|e| format!("failed to download file: {e}"))?;

            if !Self::is_windows() {
                // The archive may not preserve the executable bit.
                zed::make_file_executable(&binary_path).ok();
            }

            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::None,
            );

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            let old_version_prefix = format!("{NAME}-");
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                let filename = entry.file_name();
                let filename = filename.to_string_lossy();
                if filename.starts_with(&old_version_prefix) && filename != version_dir {
                    fs::remove_dir_all(entry.path()).ok();
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for QuarkdownExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let binary_settings = Self::user_binary_settings(worktree);

        let command = match binary_settings
            .as_ref()
            .and_then(|binary| binary.path.clone())
        {
            Some(path) => path,
            None => match worktree.which(NAME) {
                Some(path) => path,
                None => self.download_and_cache_binary(language_server_id, worktree)?,
            },
        };

        let args = binary_settings
            .as_ref()
            .and_then(|binary| binary.arguments.clone())
            .unwrap_or_else(|| vec![LANGUAGE_SERVER_ARG.to_string()]);

        let env = binary_settings
            .as_ref()
            .and_then(|binary| binary.env.clone())
            .map(Self::env_vars_from)
            .unwrap_or_default();

        Ok(Self::build_command(command, args, env))
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = zed::settings::LspSettings::for_worktree(NAME, worktree)?;
        Ok(settings.initialization_options)
    }

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<zed::serde_json::Value>> {
        let settings = zed::settings::LspSettings::for_worktree(NAME, worktree)?;
        Ok(settings.settings)
    }
}

zed::register_extension!(QuarkdownExtension);
