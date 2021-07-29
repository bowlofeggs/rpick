.PHONY: help check audit build clean clippy doc fmt test test_release

# Where the project is mounted inside the container
podman_mount_dir = /rpick
# What tag to apply to the built container
podman_tag = rpick:dev
# podman flags to mount the code into the container
podman_volume = -v $(CURDIR):$(podman_mount_dir):z -v $(CURDIR)/.cache:/root/.cargo/registry:z
# podman flags to run the container
podman_run = podman run --network=host --rm -it $(podman_volume) -e RPICK_CONFIG="$(podman_mount_dir)/config.yml" -w $(podman_mount_dir) $(podman_tag)

help:  ## Show this help
	@egrep -h '\s##\s' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

check: fmt audit test_release build clippy test doc  ## Run the full set of CI checks

audit: container  ## Run cargo audit
	$(podman_run) cargo audit

build:  ## Run cargo build
	cargo build

clean:  ## Clean build artifacts
	cargo clean || rm -rf target
	podman rmi $(podman_tag) || true
	rm -rf $(CURDIR)/.cache

clippy:  ## Run cargo clippy
	cargo clippy --all-targets --all-features -- -D warnings

container:  ## Build the container
	mkdir -p $(CURDIR)/.cache
	podman build --pull -t $(podman_tag) $(podman_volume) --force-rm=true $(CURDIR)

doc:  ## Run cargo doc
	cargo doc --no-deps

fmt:  ## Run cargo fmt
	cargo fmt -- --check -v

license: container  ## Run cargo-license
	$(podman_run) cargo license

shell: container  ## Run bash inside the development container
	$(podman_run) bash

test:  ## Run cargo test
	cargo test

test_release:  # Run cargo test --release
	cargo test --release
