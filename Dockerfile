FROM rust:bullseye as builder
WORKDIR /opt/app
COPY . .

ENV FILE_PATH="/var/rinha/source.rinha.json"

RUN rustup install nightly
RUN cargo +nightly install hvm
RUN cargo b -r

RUN cp target/release/rinha .
RUN cp $(which hvm) .

FROM bitnami/minideb
WORKDIR /opt/app

COPY --from=builder /opt/app/rinha .
COPY --from=builder /opt/app/hvm .
COPY --from=builder /opt/app/run.sh .

ENTRYPOINT ["./run.sh"]
