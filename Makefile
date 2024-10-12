build:
	cargo build

fmt:
	cargo fmt
	cargo clippy

run-example:
	SUPABASE_SERVICE_ROLE_KEY=$$(cargo run --example setup_api_key) cargo run --example basic_usage --features derive

supabase-start:
	cd supabase && supabase start
	cd supabase && supabase db reset
