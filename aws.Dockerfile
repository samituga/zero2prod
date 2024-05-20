ARG CARGO_CHEF_IMAGE

FROM ${CARGO_CHEF_IMAGE} as chef
WORKDIR /app
RUN apt update && apt install lld clang -y \
  && rustup install nightly \
  && rustup default nightly

FROM chef as planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as depbuilder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM depbuilder as builder
COPY . .
ENV SQLX_OFFLINE true
RUN rustup default nightly
RUN cargo build --release --bin zero2prod

FROM public.ecr.aws/debian/debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
