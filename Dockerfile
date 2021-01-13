FROM registry.fedoraproject.org/fedora:33
LABEL maintainer="Randy Barlow <randy@electronsweatshop.com>"

RUN dnf upgrade -y
RUN dnf install -y cargo clippy rustfmt
# This is needed for cargo-audit
RUN dnf install -y openssl-devel
RUN cargo install cargo-audit
# This is useful for finding all the licenses of the bundled libraries
RUN cargo install cargo-license

CMD ["bash"]
