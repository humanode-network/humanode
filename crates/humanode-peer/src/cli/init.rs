//! Various CLI process initialization.

use sc_cli::{CliConfiguration, SubstrateCli};

use super::CliConfigurationExt;

/// Set a custom panic handler.
pub fn set_panic_handler<T: SubstrateCli>() {
    sp_panic_handler::set(&T::support_url(), &T::impl_version());
}

/// Initialize the logger based on configuration.
pub fn init_logger<T: CliConfigurationExt>(command: &T) -> sc_cli::Result<()> {
    let substrate = command.substrate_cli_configuration();

    let mut logger = sc_cli::LoggerBuilder::new(substrate.log_filters()?);
    logger
        .with_log_reloading(substrate.enable_log_reloading()?)
        .with_detailed_output(substrate.detailed_log_output()?);

    if let Some(tracing_targets) = substrate.tracing_targets()? {
        let tracing_receiver = substrate.tracing_receiver()?;
        logger.with_profiling(tracing_receiver, tracing_targets);
    }

    if substrate.disable_log_color()? {
        logger.with_colors(false);
    }

    logger.init()?;

    Ok(())
}

/// The recommended open file descriptor limit to be configured for the process.
const RECOMMENDED_OPEN_FILE_DESCRIPTOR_LIMIT: u64 = 10_000;

/// We require a substantial amount of fds for the networking, so raise it, and report if the raised
/// limit is still way too low.
pub fn raise_fd_limit() {
    if let Ok(fdlimit::Outcome::LimitRaised { from: _, to }) = fdlimit::raise_fd_limit() {
        if to < RECOMMENDED_OPEN_FILE_DESCRIPTOR_LIMIT {
            tracing::warn!(
                "Low open file descriptor limit configured for the process. \
                Current value: {:?}, recommended value: {:?}.",
                to,
                RECOMMENDED_OPEN_FILE_DESCRIPTOR_LIMIT,
            );
        }
    }
}
