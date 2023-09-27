FROM rust:slim-bookworm as builder
WORKDIR /opt/app
COPY . .

ENV FILE_PATH="/var/rinha/source.rinha.json"

RUN rustup install nightly
RUN cargo b -r

RUN cp target/release/rinha .

FROM debian:bookworm-slim
COPY --from=builder /opt/app/rinha /
ENTRYPOINT ["/rinha"]
