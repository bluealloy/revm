.PHONY: sync_tests
sync_tests:
	if [ ! -d tests ]; then \
  		git clone https://github.com/ethereum/tests; \
  		cd tests; git pull; git checkout faf33b471465d3c6cdc3d04fbd690895f78d33f2; \
  	else \
  		cd tests; git pull; git checkout faf33b471465d3c6cdc3d04fbd690895f78d33f2; \
  	fi

.PHONY: run_general_state_tests
run_general_state_tests:
	cargo test --package revm-rwasm-e2e --bin revm-rwasm-e2e tests::good_coverage_tests

.PHONY: help
help:
	cargo run --color=always --bin revm-rwasm-e2e -h
