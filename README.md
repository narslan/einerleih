# Einerleih

## Ziel des Projektes

Einerleih ist der Start einer Verleihplattform für Gegenstände/Gemeingüter.
Nutzende kännen ohne Registrierung den Katalog durchsuchen und die Verfügbarkeit sehen können.

Das Projekt hat zwei zentrale Säulen:

- **Nutzer-Oberfläche**: Katalog + Informationsseiten (FAQ, Über Uns, Kontakt)
- **Admin-Oberfläche**: Verwaltung von Inhalten (z.B. Artikel, Zeitrahmen) 

Die Idee: ein schlankes Rust/Axum Backend (REST-API) und ein separates Lit-Frontend.

## Installation

### Voraussetzungen

- Rust/Axum (für Admin/REST-API)
- PostgreSQL 
- Node.js + `pnpm` (für das Frontend, welches sich in einem anderen [Repo](https://github.com/narslan/einerleih_ui) befindet)

### Umgebungsvariablen

Es gibt jetzt eine Vorlage unter [`.env.example`](/home/nevroz/go/src/github.com/narslan/leihladen/einerleih/.env.example).

Praktisch:

- `.env` ist für lokale Entwicklung mit `cargo run`
- dieselbe `.env` wird auch von `docker compose` für Variablenersetzung gelesen
- Secrets gehören in `.env`, nicht ins Docker-Image

Für lokalen Rust-Start sind besonders relevant:

- `LISTEN`
- `PG__URL`
- `JWT_SECRET_KEY`

Für `docker compose` sind zusätzlich relevant:

- `POSTGRES_PASSWORD`
- optional `IMAGE_REF`

Für Tests gilt getrennt davon:

- `.env.test` nutzt eine eigene Datenbank unter `PG__URL`
- empfohlen ist eine separate DB wie `einerleih_test`
- die Tests setzen das Schema dort vollständig zurück

## Release Und Deployment

Der Release-Pfad ist auf ein lokal gebautes Rust-Binary ausgelegt. Das Docker-Image baut den Rust-Code nicht selbst, sondern verpackt:

- `target/release/einerleih`
- die bereits gebauten Frontend-Dateien unter `static/`

### Lokalen Release-Build Erzeugen

```bash
cargo build --release
```

Die Binärdatei liegt danach unter:

```bash
target/release/einerleih
```

Voraussetzung für das Docker-Image:

- Das Frontend liegt bereits gebaut unter `static/dist/`
- Der Release-Build nutzt vendorte Swagger-UI-Assets und braucht dafür kein Netzwerk

### Docker-Image Lokal Bauen

```bash
./scripts/build-release-image.sh ghcr.io/narslan/einerleih:latest
```

Oder direkt:

```bash
docker build -t ghcr.io/narslan/einerleih:latest .
```

### Bei GHCR Einloggen Und Pushen

```bash
echo "$GITHUB_TOKEN" | docker login ghcr.io -u <github-user> --password-stdin
./scripts/push-ghcr.sh ghcr.io/narslan/einerleih:latest
```

### Compose Deployment

`docker-compose.yml` erwartet mindestens diese Variablen aus `.env`:

- `POSTGRES_PASSWORD`
- `JWT_SECRET_KEY`
- optional `IMAGE_REF` für ein bestimmtes GHCR-Tag

Start:

```bash
docker compose up -d
```

Wichtig:

- `docker-compose.yml` nutzt die Rust-Config-Keys wie `PG__URL` und `LISTEN`, nicht `DATABASE_URL`.
- Hochgeladene Dateien liegen im Volume `app_assets`.
- Die Datenbank liegt im Volume `pgdata`.
- Das Compose-File führt aktuell keine Datenbankmigrationen aus. Das Schema muss vorher bereitgestellt werden.


### Datenbank vorbereiten (PostgreSQL)
