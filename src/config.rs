use argh::FromArgs;

#[derive(FromArgs)]
/// Start a wasm execution server with specified config.
pub(super) struct Config {
    /// port to start the server on.
    #[argh(option, default = "4000", short = 'p')]
    pub port: u16,
    // TODO allow configuring data directory and if an memory store
}
