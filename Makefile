.PHONY: audit check clippy doc fmt fmt-fix test

CLIPPY_FLAGS := -D clippy::all -A missing_docs -A clippy::needless_borrow -A clippy::large_enum_variant
RUSTDOCFLAGS := -D rustdoc::broken-intra-doc-links -D rustdoc::private-intra-doc-links

check: fmt clippy test doc

fmt:
	cargo fmt --all --check

fmt-fix:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- $(CLIPPY_FLAGS)
	cargo clippy --all-targets --features decimal -- $(CLIPPY_FLAGS)

test:
	cargo test
	cargo test --features decimal

doc:
	RUSTDOCFLAGS="$(RUSTDOCFLAGS)" cargo doc --no-deps

audit:
	cargo audit
