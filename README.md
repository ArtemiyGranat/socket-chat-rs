# Rust TUI Socket Chat (WIP)
This is a multi-threaded client-server application that allows people to exchange information using sockets.
<!-- - [Rust TUI Socket Chat](#rust-tui-socket-chat--wip-) -->
  * [Screenshots](#screenshots)
  * [Content](#content)
  * [Dependencies](#dependencies)
  * [Setup](#setup)
  * [Usage](#usage)
    + [Client](#client)
    + [Server](#server)
  * [Features](#features)
  * [To-do](#to-do)
## Screenshots
![Server logger](/screens/server.png?raw=true "Server logger")
![Log in](/screens/log_in.png?raw=true "Log in screen")
![Log in](/screens/terminal_size.png?raw=true "Terminal size is too small")
![Log in](/screens/chat.png?raw=true "Chat")
## Content
* Folder `client` contains client source code
* Folder `server` contains server source code and Docker config files to containerize server via `docker-compose`.
## Dependencies
* Docker
* Rust
* Cargo

Make sure that you have Docker (only for the server), Rust and Cargo installed on your local machine, otherwise download it from the official site or from your package manager.
## Setup
1. Install the dependencies on your local machine
2. Create a `.env` file in the `server` folder to connect to the database. The `.env` file must contain a link to the database by which the server will connect to it. You can use the commands below with changed `<username>` and `<password>` to whatever you want.
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
Currently socket chat is in the early stage of development so now user can only connect to the server and send/receive messages to/from another users connected to the server.

Server uses custom logger and logs all connections, disconnections and requests from the clients (except data because of security), also it sends every new connection/disconnection to the clients.
## To-do
* [ ] Authentification system (WIP)
* [ ] Data encryption
* [ ] Improved Docker container (must weigh <500Mb for sure)
* [ ] Message history available to the users
* [ ] Commands system (e.g. private message: `@user hi`, get online list: `/online`)
* [ ] Rooms system
* [ ] Create Homebrew formula and Linux (e.g. RPM) package
* [ ] Share server Docker image via Docker Hub