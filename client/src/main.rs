use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

mod client;
mod ui;

use client::*;

#[tokio::main]
async fn main() {
    

    // tokio::spawn(async move {
        // });
        
    ui::run_tui().await.unwrap();
    // loop {
    //     // ui::display_message_waiting();
    //     tokio::select! {
    //         received_data = stream_reader.read_line(&mut stream_line) => {
    //             // TODO: Fix this
    //             if received_data.unwrap() == 0 {
    //                 break;
    //             }
    //             // ui::display_message(&stream_line);
    //             stream_line.clear();
    //         }
    //         _ = stdin_reader.read_line(&mut stdin_line) => {
    //             writer.write_all(&stdin_line.as_bytes()).await.unwrap();
    //             stdin_line.clear();
    //         }
    //     }
    // }
}
