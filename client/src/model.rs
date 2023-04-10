pub const SERVER_SHUTDOWN_MESSAGE: &str = "Server is shutting down, app will be closed in 10 seconds";

#[derive(Clone, Copy)]
pub(crate) enum ClientState {
    LoggingIn,
    LoggedIn,
}

#[derive(Clone, Copy)]
pub(crate) enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
pub(crate) enum Command {
    Exit,
    SendMessage(String),
    LogInUsername(String),
}