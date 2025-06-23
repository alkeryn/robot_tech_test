# Base image that will be reused
FROM rust:1.86 as base

FROM base as builder
WORKDIR /usr/src/build
COPY . .
RUN cargo build --release

FROM base as runtime
WORKDIR /app
COPY --from=builder /usr/src/build/target/release/robot_tech_test /app/
CMD ["./robot_tech_test"]
