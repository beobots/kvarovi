build: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
build:
	sam build --beta-features --debug

deploy: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
deploy:
	sam deploy --beta-features --debug

list: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
list:
	sam list resources

delete: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
delete:
	sam delete --beta-features --debug

clean:
	cargo clean
	rm -rf ./.aws-sam

.PHONY: delete clean list deploy build

download-streets:
	target/debug/download_beo_streets | tee download.csv && sort download.csv | uniq | tr '[:upper:]' '[:lower:]' > beograd_streets/beograd_streets.csv
