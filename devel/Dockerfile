FROM registry.fedoraproject.org/fedora:31
LABEL maintainer="Randy Barlow <bowlofeggs@fedoraproject.org>"

RUN dnf install -y cargo clippy
# This way we can cache all the build dependencies
RUN cd /project && cargo build
# This way we can cache all the development dependencies
RUN cd /project && cargo test
# This way we can cache all the doc dependencies
RUN cd /project && cargo doc
# This is needed for cargo-audit
RUN dnf install -y openssl-devel
RUN cargo install cargo-audit
# This is useful for finding all the licenses of the bundled libraries
RUN cargo install cargo-license

CMD ["bash"]
