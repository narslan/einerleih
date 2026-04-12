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

### Datenbankmigrationen

Die SQL-Migrationen liegen unter [`db/migrations`](/home/nevroz/go/src/github.com/narslan/leihladen/einerleih/db/migrations).
Optionale Entwicklungs-Seed-Daten liegen unter [`db/seeds`](/home/nevroz/go/src/github.com/narslan/leihladen/einerleih/db/seeds).

Verfügbare Kommandos:

```bash
cargo run -- migrate up
cargo run -- migrate down
cargo run -- migrate reset
cargo run -- migrate status
cargo run -- seed
RUST_LOG=info,tokio_postgres=debug,tower_http=debug cargo run
```

Der normale Serverstart führt `migrate up` automatisch aus, bevor der HTTP-Server startet.
Frontend-relevante Basisdaten für `towns` und `categories` werden ebenfalls über Migrationen angelegt und stehen damit auch nach einem Produktions-Deployment automatisch bereit. Der `seed`-Befehl bleibt für zusätzliche Development-Daten gedacht.

### Einmaliger destruktiver Reset im Deployment

Die Produktionsdatenbank vor dem Backend-Start explizit neu aufgebaut werden:

```bash
docker compose up -d postgres
docker compose stop backend
docker compose --profile db-reset run --rm migrate-reset
docker compose up -d backend
```

Den Reset nicht in den normalen Backend-Start einbauen: Ein Container-Restart oder erneutes `docker compose up -d` dürfte sonst später echte Daten löschen.

### Temporärer Bootstrap-Admin

Für frühe Deployments kann der Backend-Start ein Admin-Konto erzeugen oder aktualisieren, wenn diese Variablen gesetzt sind:

```bash
BOOTSTRAP_ADMIN_USERNAME=admin
BOOTSTRAP_ADMIN_EMAIL=admin@example.com
BOOTSTRAP_ADMIN_PASSWORD=<starkes-passwort>
```

Das Passwort wird nicht gespeichert, sondern beim Start gehasht. Sobald wir eine endgültige Admin-Verwaltung haben, sollten diese Variablen wieder entfernt werden.
