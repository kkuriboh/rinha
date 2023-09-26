FROM rust:slim-bookworm as builder
WORKDIR /opt/app
COPY . .

ENV FILE_PATH="/var/rinha/source.rinha.json"

RUN rustup install nightly
RUN cargo +nightly install hvm
RUN cargo b -r

RUN cp target/release/rinha .
RUN cp $(which hvm) .

FROM debian:bookworm-slim

COPY --from=builder /opt/app/rinha /
COPY --from=builder /opt/app/hvm /

CMD dash -c "./rinha && ./hvm run -f main.hvm"
