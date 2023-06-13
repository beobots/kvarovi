build: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
build:
	sam build --beta-features --debug

deploy: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
deploy:
	sam deploy --beta-features --debug


list: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
list:
	sam list resources

clean:
	cargo clean

delete: SAM_CLI_BETA_RUST_CARGO_LAMBDA=1
delete: clean
	sam delete --beta-features --debug
