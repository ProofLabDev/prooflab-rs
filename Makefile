# SHELL := $(shell echo $$SHELL)

# # Find the profile directory based on the shell
# ifeq ($(findstring bash,$(SHELL)),bash)
#     PROFILE := ~/.bashrc
# else ifeq ($(findstring zsh,$(SHELL)),zsh)
#     PROFILE := ~/.zshenv
# else ifeq ($(findstring fish,$(SHELL)),fish)
#     PROFILE := ~/.config/fish/config.fish
# else ifeq ($(findstring sh,$(SHELL)),sh)
#     PROFILE := ~/.profile
# else
#     echo "prooflab-rs: could not detect shell, manually add ${PROOFLAB_BIN_DIR} to your PATH."
# 	exit 1
# endif

install: install_prooflab

install_prooflab:
	@curl -L https://raw.githubusercontent.com/ProofLabDev/prooflab-rs/main/install_prooflab.sh | bash
	
build:
	@./build.sh

check:
	cargo clippy --all -- -D warnings

install_sp1:
	@curl -L https://sp1.succinct.xyz | bash
	@source $(PROFILE)
	@sp1up
	@cargo prove --version

install_risc0:
	@curl -L https://risczero.com/install | bash
	@rzup install
	@cargo risczero --version

install_jolt:
	@echo "Installing Jolt from source"
	@cargo install --git https://github.com/a16z/jolt jolt-cli

all: install

__EXAMPLES__:

# RISC0
prove_risc0_fibonacci:
	cargo run --release -p prooflab -- prove-risc0 examples/fibonacci

prove_risc0_rsa:
	cargo run --release -p prooflab -- prove-risc0 examples/rsa

prove_risc0_ecdsa:
	cargo run --release -p prooflab -- prove-risc0 examples/ecdsa

prove_risc0_json:
	cargo run --release -p prooflab -- prove-risc0 examples/json

prove_risc0_regex:
	cargo run --release -p prooflab -- prove-risc0 examples/regex

prove_risc0_sha:
	cargo run --release -p prooflab -- prove-risc0 examples/sha

prove_risc0_tendermint:
	cargo run --release -p prooflab -- prove-risc0 examples/tendermint

prove_risc0_zkquiz:
	cargo run --release -p prooflab -- prove-risc0 examples/zkquiz

prove_risc0_bubble_sort:
	cargo run --release -p prooflab -- prove-risc0 examples/bubble_sort

# SP1
prove_sp1_fibonacci:
	cargo run --release -p prooflab -- prove-sp1 examples/fibonacci

prove_sp1_rsa:
	cargo run --release -p prooflab -- prove-sp1 examples/rsa

prove_sp1_ecdsa:
	cargo run --release -p prooflab -- prove-sp1 examples/ecdsa

prove_sp1_eddsa:
	cargo run --release -p prooflab -- prove-sp1 examples/eddsa

prove_sp1_keccak256:
	cargo run --release -p prooflab -- prove-sp1 examples/keccak256

prove_sp1_json:
	cargo run --release -p prooflab -- prove-sp1 examples/json

prove_sp1_regex:
	cargo run --release -p prooflab -- prove-sp1 examples/regex

prove_sp1_sha:
	cargo run --release -p prooflab -- prove-sp1 examples/sha

prove_sp1_tendermint:
	cargo run --release -p prooflab -- prove-sp1 examples/tendermint

prove_sp1_zkquiz:
	cargo run --release -p prooflab -- prove-sp1 examples/zkquiz

prove_sp1_iseven:
	cargo run --release -p prooflab -- prove-sp1 examples/is_even

prove_sp1_bubble_sort:
	cargo run --release -p prooflab -- prove-sp1 examples/bubble_sort

# JOLT
prove_jolt_fibonacci:
	cargo run --release -p prooflab -- prove-jolt examples/fibonacci

prove_jolt_is_even:
	cargo run --release -p prooflab -- prove-jolt examples/is_even

prove_jolt_bubble_sort:
	cargo run --release -p prooflab -- prove-jolt examples/bubble_sort

# Benchmark Commands
benchmark_sp1_fibonacci:
	cargo run --release -p prooflab -- prove-sp1 examples/fibonacci --enable-telemetry

benchmark_sp1_rsa:
	cargo run --release -p prooflab -- prove-sp1 examples/rsa --enable-telemetry

benchmark_sp1_ecdsa:
	cargo run --release -p prooflab -- prove-sp1 examples/ecdsa --enable-telemetry

benchmark_sp1_eddsa:
	cargo run --release -p prooflab -- prove-sp1 examples/eddsa --enable-telemetry

benchmark_sp1_keccak256:
	cargo run --release -p prooflab -- prove-sp1 examples/keccak256 --enable-telemetry

benchmark_sp1_json:
	cargo run --release -p prooflab -- prove-sp1 examples/json --enable-telemetry

benchmark_sp1_regex:
	cargo run --release -p prooflab -- prove-sp1 examples/regex --enable-telemetry

benchmark_sp1_sha:
	cargo run --release -p prooflab -- prove-sp1 examples/sha --enable-telemetry

benchmark_sp1_tendermint:
	cargo run --release -p prooflab -- prove-sp1 examples/tendermint --enable-telemetry

benchmark_sp1_zkquiz:
	cargo run --release -p prooflab -- prove-sp1 examples/zkquiz --enable-telemetry

benchmark_sp1_iseven:
	cargo run --release -p prooflab -- prove-sp1 examples/is_even --enable-telemetry

benchmark_sp1_bubble_sort:
	cargo run --release -p prooflab -- prove-sp1 examples/bubble_sort --enable-telemetry


benchmark_risc0_fibonacci:
	cargo run --release -p prooflab -- prove-risc0 examples/fibonacci --enable-telemetry

benchmark_risc0_rsa:
	cargo run --release -p prooflab -- prove-risc0 examples/rsa --enable-telemetry

benchmark_risc0_ecdsa:
	cargo run --release -p prooflab -- prove-risc0 examples/ecdsa --enable-telemetry

benchmark_risc0_json:
	cargo run --release -p prooflab -- prove-risc0 examples/json --enable-telemetry

benchmark_risc0_regex:
	cargo run --release -p prooflab -- prove-risc0 examples/regex --enable-telemetry

benchmark_risc0_sha:
	cargo run --release -p prooflab -- prove-risc0 examples/sha --enable-telemetry

benchmark_risc0_tendermint:
	cargo run --release -p prooflab -- prove-risc0 examples/tendermint --enable-telemetry

benchmark_risc0_zkquiz:
	cargo run --release -p prooflab -- prove-risc0 examples/zkquiz --enable-telemetry

benchmark_risc0_iseven:
	cargo run --release -p prooflab -- prove-risc0 examples/is_even --enable-telemetry

benchmark_risc0_bubble_sort:
	cargo run --release -p prooflab -- prove-risc0 examples/bubble_sort --enable-telemetry

# Docker commands
docker-shell:
	docker run -it \
		--platform=linux/amd64 \
		-v prooflab-rs-cargo-registry:/root/.cargo/registry \
		-v prooflab-rs-cargo-git:/root/.cargo/git \
		-v "$(PWD)/crates:/prooflab-rs/crates" \
		-v "$(PWD)/telemetry:/prooflab-rs/telemetry" \
		-v "$(PWD)/Makefile:/prooflab-rs/Makefile" \
		-v "$(PWD)/examples:/prooflab-rs/examples" \
		-w /prooflab-rs \
		prooflab-rs bash

docker-build:
	DOCKER_BUILDKIT=1 docker build --platform=linux/amd64 -t prooflab-rs -f Dockerfile.cpu .