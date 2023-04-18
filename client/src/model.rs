pub const SERVER_SHUTDOWN_MESSAGE: &str = "Server is shutting down, app will be closed in 10 seconds";

#[derive(Clone, Copy)]
pub(crate) enum Stage {
    Username,
    Password,
    Choosing,
}

#[derive(Clone, Copy)]
pub(crate) enum ClientState {
    LoggedIn,
    LoggingIn(Stage),
    Registering(Stage),
}

#[derive(Clone, Copy)]
pub(crate) enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
pub(crate) enum Request {
    Exit,
    SendMessage(String),
    LogInUsername(String),
    LogInPassword(String),
}