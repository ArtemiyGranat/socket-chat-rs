# Rust TUI Socket Chat (WIP)
This is a multithreaded client-server application that allows people to exchange information using sockets. 

* [Screenshots](#screenshots)
* [Content](#content)
* [Dependencies](#dependencies)
* [Setup](#setup)
* [Usage](#usage)
    + [Client](#client)
    + [Server](#server)
* [Features](#features)
* [To-do](#to-do)
* [Additional screenshots](#additional-screenshots)

## Screenshots
![Log in](/screens/chat.png?raw=true "Chat")
![Server logger](/screens/server.png?raw=true "Server logger")
## Content
* The `client` folder contains the client source code
* The `server` folder contains the server source code and Docker configuration files for server containerization using `docker-compose`.
## Dependencies
* Docker
* Rust
* Cargo

Make sure you have Docker (server only), Rust and Cargo installed on your local machine, otherwise download it from the official site or from your package manager.
## Setup
1. Install the dependencies on your local machine
2. Create a `.env` file in the `server` folder to connect to the database. The `.env` file must contain a link to the database that the server will use to connect to it. You can use the commands below with `<username>` and `<password>` changed to whatever you want.
```
cd server
touch .env
echo DATABASE_URL=postgres://<username>:<password>@0.0.0.0:5432/socket-chat-db > .env
```
3. Change the `POSTGRES_USER` and `POSTGRES_PASSWORD` fields to the `<username>` and `<password>` from the previous step and make sure they are matching.
## Usage
### Client
```
cd client
cargo run --release
```
### Server
```
cd server
docker compose-up -d
cargo run --release
```
## Features
Socket chat is currently at an early stage of development, so for now the user can only connect to the server and exchange messages with other users connected to the server.

The server uses a custom logger and logs all connections, disconnections and requests from clients (except received data due to security), and sends each new connection / disconnection to the clients.
## To-do
* [ ] Authentification system (WIP)
* [ ] Data encryption
* [ ] Improved Docker container (must be <500Mb, WIP)
* [ ] Message history available to the users
* [ ] Commands system (e.g. private message: `@user hi`, get online list: `/online`)
* [ ] Rooms system
* [ ] Homebrew formula and Linux (e.g. RPM) package
* [ ] Push Docker image to Docker Hub
## Additional screenshots
![Log in](/screens/log_in.png?raw=true "Log in screen")
![Log in](/screens/terminal_size.png?raw=true "Terminal size is too small")