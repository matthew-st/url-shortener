FROM rust:1.56 as builder
EXPOSE 8000
WORKDIR /usr/src/url-shortener
COPY . .
RUN cargo build --release
ENV ROCKET_ADDRESS=0.0.0.0
CMD ["./target/release/url-shortener"]