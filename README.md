Conventions:
.yaml

dependencies:

- rust and cargo (cargo install)
    - cargo-clip
    - cargo-sort
    - sqlx-cli
- pre-commit:
    - https://pre-commit.com/
- psql

```shell
# linux
sudo apt update & sudo apt upgrade
sudo apt install postgresql postgresql-contrib
# macOS (brew)
brew install postgresql
```

pre-commit

```shell
#linux
pip install pre-commit

#macOS (brew)
brew install pre-commit
```

Format code using rust formatter

```shell
cargo fmt
```

Sort dependencies

```shell
cargo sort
```

Generate offline sql data (needs postgres to be running)\
sqlx calls into our database at compile-time to ensure that all queries can be successfully executed considering
the schemas of our tables.\
We use this command locally to save the results so that we don't need a live connection on the CI pipelines.\
We will need the env variable `SQLX_OFFLINE=true` to use the offline data

```shell
cargo sqlx prepare --workspace
```

Run pre commit validations to all files (runs automatically before each commit but only for changed files)

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
Every job keeps downloading and compiling the rust dependencies, check if rust cache is working
Consider running everything in a single job

TODO Infra
Just by adding a new dependency the cache does not work as expected, it downloads and compiles all dependencies
Configure AWS CodeDeploy
Configure AWS secret manager for secret env vars like DB password
Use HCP vault for secrets
