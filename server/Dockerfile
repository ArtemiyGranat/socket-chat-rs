FROM rust:1.68.0

WORKDIR /app

COPY . .

RUN cargo build --release

EXPOSE 8080

CMD ["cargo", "run", "--release"]