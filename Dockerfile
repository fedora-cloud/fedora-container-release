FROM fedora:latest

RUN dnf update -y && dnf install -y cargo openssl openssl-devel xz
WORKDIR /code
ENV PATH="${PATH}:/root/.cargo/bin"
ADD . /code
RUN cargo clean && cargo update && cargo build --release
RUN cargo install --path .

ENTRYPOINT ["fedora-container-release", "--release"]
