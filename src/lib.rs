pub mod add;
pub mod init;
pub mod lock;
pub mod megacmd;
pub mod options;
pub mod push;
pub mod remove;
pub mod status;

pub use add::*;
pub use init::*;
pub use push::*;
pub use remove::*;
pub use status::*;

pub const OPTIONS_PATH: &'static str = ".mega.toml";
pub const LOCK_PATH: &'static str = ".mega.lock";
