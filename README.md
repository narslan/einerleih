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
Optionale Seed-Daten liegen unter [`db/seeds`](/home/nevroz/go/src/github.com/narslan/leihladen/einerleih/db/seeds).

Verfügbare Kommandos:

```bash
cargo run -- migrate up
cargo run -- migrate down
cargo run -- migrate reset
cargo run -- migrate status
cargo run -- seed
```

Der normale Serverstart führt `migrate up` automatisch aus, bevor der HTTP-Server startet.
