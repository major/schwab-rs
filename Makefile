.PHONY: audit check clean clippy coverage doc fmt fmt-fix machete patch-coverage test

CLIPPY_FLAGS := -D clippy::all -A clippy::needless_borrow -A clippy::large_enum_variant
RUSTDOCFLAGS := -D rustdoc::broken-intra-doc-links -D rustdoc::private-intra-doc-links
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

test:
	cargo test
	cargo test --features decimal

doc:
	RUSTDOCFLAGS="$(RUSTDOCFLAGS)" cargo doc --no-deps

coverage:
	cargo llvm-cov test --fail-under-lines 90

patch-coverage:
	cargo llvm-cov --workspace --fail-under-lines 90 --lcov --output-path lcov.info
	$(DIFF_COVER) lcov.info --compare-branch=$(PATCH_COVERAGE_BASE) --fail-under=$(PATCH_COVERAGE_FAIL_UNDER)

audit:
	cargo audit

machete:
	cargo machete

clean:
	cargo clean
	rm -f lcov.info
