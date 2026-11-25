# Zero-install ARRAKIS: everything builds and runs inside the container.
#   docker build -t arrakis .
#   docker run --rm arrakis                    # the quickstart (all headline demos)
#   docker run --rm arrakis chaos --seed 42    # any ./arrakis.sh command
#   docker run --rm arrakis tour               # component demos (builds each workspace on first use)
FROM rust:1-slim
WORKDIR /arrakis
COPY . .
RUN chmod +x arrakis.sh \
 && cargo build --release --manifest-path integration/Cargo.toml --bin arrakis
ENTRYPOINT ["./arrakis.sh"]
CMD ["quickstart"]
