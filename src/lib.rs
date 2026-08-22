use std::fs;

use zed_extension_api::{
    self as zed, Architecture, LanguageServerId, Os, Result, current_platform,
};

const NAME: &str = "quarkdown";
const REPO: &str = "iamgio/quarkdown";

const BIN_DIRECTORY: &str = "quarkdown/bin/quarkdown";

const ASSET_LINUX: &str = "quarkdown-linux-x64.zip";
const ASSET_WINDOWS: &str = "quarkdown-windows-x64.zip";
const ASSET_MACOS_SILICON: &str = "quarkdown-macos-aarch64.zip";
const ASSET_MACOS_INTEL: &str = "quarkdown-macos-x64.zip";

struct QuarkdownExtension {
    cached_binary_path: Option<String>,
}

impl QuarkdownExtension {
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
            .ok_or_else(|| format!("no asset found matching {:?}", asset_name))?;

        let version_dir = format!("{NAME}-{}", release.version);
        let binary_path = format!("{version_dir}/{BIN_DIRECTORY}");

        if !fs::metadata(&binary_path).is_ok_and(|stat| stat.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::download_file(&asset.download_url, &version_dir, download_type)
                .map_err(|e| format!("failed to download file: {e}"))?;

            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::None,
            );

            let entries =
                fs::read_dir(".").map_err(|e| format!("failed to list working directory {e}"))?;
            for entry in entries {
                let entry = entry.map_err(|e| format!("failed to load directory entry {e}"))?;
                let filename = entry.file_name();
                let filename = filename.to_str().unwrap();
                if filename.starts_with("{NAME}-") && filename != version_dir {
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
        let command = match worktree.which(NAME) {
            Some(path) => path,
            None => self.download_and_cache_binary(language_server_id, worktree)?,
        };

        Ok(zed::Command {
            command,
            args: vec!["language-server".to_string()],
            env: Default::default(),
        })
    }
}

zed::register_extension!(QuarkdownExtension);
