#[macro_export]
macro_rules! conn_message {
    ($username:expr) => {{
        let conn_message = format!("{} has been connected to the server\n", $username);
        print!(
            "[{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            conn_message
        );
        conn_message
    }};
}

#[macro_export]
macro_rules! disc_message {
    ($username:expr) => {{
        let disc_message = format!("{} has been disconnected from the server\n", $username);
        print!(
            "[{}] {}",
            Local::now().format("%Y-%m-%d %H:%M:%S"),
            disc_message
        );
        disc_message
    }};
}

#[macro_export]
macro_rules! print_message {
    ($username:expr, $data:expr) => {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S");
        print!("[{}] [{}] {}", now, $username, $data)
    };
}
