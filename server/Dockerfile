FROM rustlang/rust:nightly-slim

WORKDIR /app

COPY . .

RUN cargo install sqlx-cli --no-default-features --features rustls,postgres
RUN sqlx migrate run
RUN cargo +nightly build --release

EXPOSE 8080

CMD ["cargo", "run", "--release"]