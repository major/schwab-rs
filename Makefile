.PHONY: audit check clean clippy coverage doc fmt fmt-fix machete patch-coverage test

CLIPPY_FLAGS := -D clippy::all -A clippy::needless_borrow -A clippy::large_enum_variant
RUSTDOCFLAGS := -D rustdoc::broken-intra-doc-links -D rustdoc::private-intra-doc-links
COVERAGE_RUSTFLAGS := --cfg coverage_nightly
PATCH_COVERAGE_BASE ?= main
PATCH_COVERAGE_FAIL_UNDER ?= 100
DIFF_COVER ?= diff-cover

check: fmt clippy test doc

fmt:
	cargo fmt --all --check

fmt-fix:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- $(CLIPPY_FLAGS)
	cargo clippy --all-targets --features decimal -- $(CLIPPY_FLAGS)
	cargo clippy --lib --no-default-features -- $(CLIPPY_FLAGS)
	cargo clippy --lib --no-default-features --features decimal -- $(CLIPPY_FLAGS)

test:
	cargo test
	cargo test --features decimal
	cargo test --lib --no-default-features
	cargo test --lib --no-default-features --features decimal

doc:
	RUSTDOCFLAGS="$(RUSTDOCFLAGS)" cargo doc --no-deps
	RUSTDOCFLAGS="$(RUSTDOCFLAGS)" cargo doc --lib --no-default-features --no-deps

coverage:
	RUSTFLAGS="$(COVERAGE_RUSTFLAGS)" cargo +nightly llvm-cov test --fail-under-lines 90

patch-coverage:
	RUSTFLAGS="$(COVERAGE_RUSTFLAGS)" cargo +nightly llvm-cov --workspace --fail-under-lines 90 --lcov --output-path lcov.info
	$(DIFF_COVER) lcov.info --compare-branch=$(PATCH_COVERAGE_BASE) --fail-under=$(PATCH_COVERAGE_FAIL_UNDER)

audit:
	cargo audit

machete:
	cargo machete

clean:
	cargo clean
	rm -f lcov.info
