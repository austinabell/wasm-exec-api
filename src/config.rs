use argh::FromArgs;

#[derive(FromArgs)]
/// Start a wasm execution server with specified config.
pub(super) struct Config {
    /// port to start the server on.
    #[argh(option, default = "4000", short = 'p')]
    pub port: u16,

    /// data directory for storing registered Wasm functions.
    #[cfg(not(feature = "p2p"))]
    #[argh(option, short = 'd')]
    pub data_directory: Option<String>,

    /// if flag is set, database will not be persisted
    #[cfg(not(feature = "p2p"))]
    #[argh(switch, short = 'm')]
    pub memory: bool,
}
