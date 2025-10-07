# Setup

In [draid-binary](./draid-binary):
* `cargo install sqlx-cli --no-default-features --features native-tls,postgres`
* `cargo sqlx migrate add users --source migrations`
* `cargo sqlx migrate add sessions --source migrations`
* `cargo sqlx migrate add messages --source migrations`
* `cargo sqlx migrate add roles --source migrations`
* `cargo sqlx migrate add traces --source migrations`
* `DATABASE_URL="postgres://postgres:yourpassword@localhost:5432/draid" cargo sqlx database create`
* `DATABASE_URL="postgres://postgres:yourpassword@localhost:5432/draid" cargo sqlx migrate run --source migrations`
* `DATABASE_URL="postgres://postgres:yourpassword@localhost:5432/kb" cargo sqlx migrate run --source migrations-vector`
* `DATABASE_URL="postgres://postgres:yourpassword@localhost:5432/draid" cargo sqlx prepare`
* `DATABASE_URL="postgres://postgres:yourpassword@localhost:5432/kb" cargo sqlx prepare`
ROCKET_DATABASES='{kb={url="postgresql://postgres:yourpassword@localhost:5432/kb"}}'
