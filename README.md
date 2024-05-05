Conventions:
.yaml over yml

dependencies:
psql

```shell
sudo apt update & sudo apt upgrade
sudo apt install postgresql postgresql-contrib
```

Format code using rust formatter

```shell
cargo fmt
```

Sort dependencies

```shell
cargo sort
```

Generate offline sql data

```shell
cargo sqlx prepare --workspace
```

Run pre commit validations (runs automatically before each commit)

```shell
pre-commit run --all-files
```

run init db script

```shell
./scripts/init_db.sh
SKIP_DOCKER=true ./scripts/init_db.sh # If we want to skip docker initialization
```

Build docker image

```shell
docker build --tag zero2pod --file Dockerfile .
```

TODO GitHub workflows
Docker caching doesn't appear to be working
Every job keeps downloading and compiling the rust dependencies, checko if rust cache is working
Consider running everything in a single job
