use zed::settings::LspSettings;
use zed_extension_api::{self as zed, LanguageServerId, Result};

const SERVER_NAME: &str = "ultrareview-bridge";
const DEFAULT_PORT: &str = "19999";

struct UltrareviewBridgeExtension;

impl UltrareviewBridgeExtension {
    fn default_args() -> Vec<String> {
        vec![
            "lsp".to_string(),
            "--port".to_string(),
            DEFAULT_PORT.to_string(),
        ]
    }
}

impl zed::Extension for UltrareviewBridgeExtension {
    fn new() -> Self {
        Self
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        let settings = LspSettings::for_worktree(language_server_id.as_ref(), worktree)?;
        let mut args = Self::default_args();

        if let Some(binary) = settings.binary {
            let env = binary
                .env
                .unwrap_or_default()
                .into_iter()
                .collect::<Vec<_>>();

            return Ok(zed::Command {
                command: binary.path.unwrap_or_else(|| SERVER_NAME.to_string()),
                args: binary.arguments.unwrap_or(args),
                env,
            });
        }

        let command = worktree
            .which(SERVER_NAME)
            .unwrap_or_else(|| SERVER_NAME.to_string());

        Ok(zed::Command {
            command,
            args: std::mem::take(&mut args),
            env: Default::default(),
        })
    }
}

zed::register_extension!(UltrareviewBridgeExtension);
