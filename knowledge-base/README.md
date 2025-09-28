# Setup

* `cargo install sqlx-cli --no-default-features --features native-tls,postgres`
* `cargo sqlx migrate add vectors --source migrations`
* `cargo sqlx migrate add documents --source migrations`
* `DATABASE_URL="postgres://postgres:yourpassword@localhost:5432/kb" cargo sqlx database create`
* `DATABASE_URL="postgres://postgres:yourpassword@localhost:5432/kb" cargo sqlx migrate run --source migrations`
* `DATABASE_URL="postgres://postgres:yourpassword@localhost:5432/kb" cargo sqlx prepare`

#
