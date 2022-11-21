//! The command to export embedded runtime WASM.

use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

/// The command.
#[derive(Debug, clap::Parser)]
pub struct ExportEmbeddedRuntimeCmd {
    /// Specify the output path.
    #[clap(long, short = 'o')]
    out: Option<PathBuf>,
}

impl ExportEmbeddedRuntimeCmd {
    /// Run the export embedded runtime command.
    pub async fn run(&self) -> sc_cli::Result<()> {
        let data = humanode_runtime::WASM_BINARY.ok_or_else(|| {
            sc_cli::Error::Application("WASM binary is not embedded in this build".into())
        })?;

        match self.out {
            Some(ref path) => tokio::fs::write(path, data).await,
            None => tokio::io::stdout().write_all(data).await,
        }
        .map_err(sc_cli::Error::Io)?;

        Ok(())
    }
}
