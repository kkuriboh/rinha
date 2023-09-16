FROM rust:alpine as builder
WORKDIR /opt/app
COPY . .

RUN cargo b -r
RUN cp target/release/rinha .

FROM alpine
COPY --from=builder /opt/app/rinha .
CMD ["./rinha", "/var/rinha/source.rinha.json"]
