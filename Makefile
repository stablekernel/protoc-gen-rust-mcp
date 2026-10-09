.PHONY: protoc-gen-rust-mcp generate fmt

PROTOS := $(shell find ./examples/protos -name '*.proto')
GENDIR := examples/src/gen

# protoc 29.3, protoc-gen-prost 0.5.0, protoc-gen-tonic 0.5.0 and
# protoc-gen-prost-serde 0.4.0 are expected on PATH (see AGENTS.md and
# Dockerfile.factory).

fmt:
	@echo "Formatting files..."
	@cargo fmt --all && echo "Formatted successfully!"

protoc-gen-rust-mcp:
	@echo "Building protoc-gen-rust-mcp..."
	@cargo build -p protoc-gen-rust-mcp && echo "Built successfully!"

generate: protoc-gen-rust-mcp
	@echo "Generating new files..."
	@rm -rf $(GENDIR)
	@mkdir -p $(GENDIR)
	@protoc \
		--proto_path=examples/protos \
		--prost_out=$(GENDIR) \
		--tonic_out=$(GENDIR) \
		--prost-serde_out=$(GENDIR) \
		--plugin=protoc-gen-rust-mcp=target/debug/protoc-gen-rust-mcp \
		--rust-mcp_out=$(GENDIR) \
		$(PROTOS) && echo "Generated successfully!"
