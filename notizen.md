Examples:
https://github.com/deadpool-rs/deadpool/blob/main/examples/postgres-axum/src/main.rs
https://github.com/microsoft/RustTraining/blob/main/async-book/src/ch02-the-future-trait.md

```sql
DROP SCHEMA public CASCADE;
CREATE SCHEMA public;
GRANT ALL ON SCHEMA public TO postgres;
GRANT ALL ON SCHEMA public TO public;


```
```sh
cargo build --release
  ./scripts/build-release-image.sh ghcr.io/narslan/einerleih:latest
  echo $GITHUB_TOKEN |  docker push ghcr.io/narslan/einerleih:latest
  #echo "$GITHUB_TOKEN" | docker login ghcr.io -u <github-user> --password-stdin
  oder
  ./scripts/push-ghcr.sh ghcr.io/narslan/einerleih:latest
```
