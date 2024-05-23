# Rust Web Example

> Written by: Nick Giampietro
> AKA `giampiet` (PSU)
> AKA `jnickg` (General)

## Overview

A collection of random one-off explorations into Rust-basd full-stack development

## Developer Guide

### Building

Create binaries and docker images needed to run the server

```bash
rustup override set nightly # required for test infra
cargo build --release
docker compose build mongodb
```

### Running

Set up admin user credentials for the MongoDB container

```bash
echo "myPassword" > server/secrets/mongo-pw.txt
echo "myAdminUser" > server/secrets/mongo-user.txt
# This is required because docker compose and/or official MongoDB image is jank and won't honor
# MONGO_INITDB_ROOT_PASSWORD_FILE environment variables, just MONGO_INITDB_ROOT_PASSWORD. To keep
# the passwords out of source control (out of docker-compose.yaml), we make an env file for the
# mongo image, which seems to work
./generate_mongo_env.sh
docker compose up --build -d mongodb --force-recreate
cargo run -- --host localhost --user myAdminUser --pass ./server/secrets/mongo-pw.txt --port 27017
```

### Using

- See the [`./examples`](./examples/) directory for some examples of interacting with the server, including `curl` commands
- Navigate to [http://localhost:3000](http://localhost:3000) for a landing page, which includes lings to OpenAPI documentation and the like

### Cleaning

Clean the MongoDB instance of all data

```bash
docker compose down mongodb
sudo rm -rf ./mongo/db # We volume mount DB data so it persists between sessions. This clears local files
```

### Tasks

- [x] Image support (CRUD)
  - [x] Format conversion for all supported [`ImageFormat` mappings](https://docs.rs/image/latest/image/enum.ImageFormat.html#variants) (GET with `Content-Type` header and/or file extension in path)
- [x] Matrix support (CRUD)
- [x] Matrix Math REST interface (dot product, add, subtract)
- [x] Image filtering/convolution with arbitrary kernel
- [x] Image Pyramid generation (Gaussian filter + strided subsampling)
- [x] Pyramid Tile generation ($\text{512}\times\text{512}$)
- [x] CLI tool for pyramid/tile generation
- [x] Brotli compression of tiled image pyramid
- [x] Persistent DB backend (MongoDB)
  - [x] integrate Image support (Doc + GridFS)
  - [ ] integrate Matrix support (Doc)
  - [x] integrate Pyramid support (Doc + Images)
  - [x] integrate Tile support (Doc + Images)
- [ ] Wasm support
  - [ ] Headless backend
  - [ ] In-browser frontend - follow [this](https://robert.kra.hn/posts/2022-04-03_rust-web-wasm/)
- [ ] Websocket support for long-running tasks
- [ ] User interface

## Support

Open an [issue](https://github.com/jnickg/rust-explorations/issues) with a question or bug report, or feel free to open a [pull request](https://github.com/jnickg/rust-explorations/pulls).

## Credits

- The glorious and infallable [`knock-knock`](https://github.com/pdx-cs-rust-web/knock-knock) repo was used as inspiration in terms of structure and crates used. Some code (especially middleware setup) may be similar.
