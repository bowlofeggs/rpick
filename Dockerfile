FROM registry.fedoraproject.org/fedora:36
LABEL maintainer="Randy Barlow <randy@electronsweatshop.com>"

RUN dnf upgrade -y
# openssl-devel is needed for cargo-audit
RUN dnf install -y cargo clippy openssl-devel rustfmt
# cargo-license is useful for finding all the licenses of the bundled libraries
RUN cargo install cargo-audit cargo-license

CMD ["bash"]
