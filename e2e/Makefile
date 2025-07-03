.PHONY: sync_tests
sync_tests:
	if [ ! -d tests ]; then \
  		git clone https://github.com/ethereum/tests; \
  	fi
	cd tests; git pull; git checkout v16.0; \

.PHONY: run_general_state_tests
run_general_state_tests:
	cargo test --release --package revm-rwasm-e2e --bin revm-rwasm-e2e tests::good_coverage_tests

.PHONY: run_all_tests
run_all_tests:
	cargo test --release --package revm-rwasm-e2e --bin revm-rwasm-e2e

.PHONY: help
help:
	cargo run --color=always --bin revm-rwasm-e2e -h
