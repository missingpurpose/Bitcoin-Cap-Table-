ENDPOINT ?= bitcoin.firehose.pinax.network:443

.PHONY: build
build:
	cargo build --target wasm32-unknown-unknown --release

.PHONY: stream
stream: build
	substreams run -e $(ENDPOINT) substreams.yaml store_account_holdings -s 620000 -t +500 --debug-modules-output

.PHONY: tt
tt: 
	substreams run -e $(ENDPOINT) substreams.yaml graph_out -s 620000 -t +2000 -o json

.PHONY: codegen
codegen:
	substreams protogen ./substreams.yaml --exclude-paths="sf/substreams,google"

.PHONY: package
package:
	substreams pack ./substreams.yaml