# Einerleih

## Ziel des Projektes

Einerleih ist der Start einer Verleihplattform für Gegenstände/Gemeingüter.
Ziel ist es, dass Nutzende ohne Registrierung den Katalog durchsuchen und die Verfügbarkeit sehen können.

Das Projekt hat zwei zentrale Säulen:

- **Nutzer-Oberfläche**: Katalog + Informationsseiten (FAQ, Über Uns, Kontakt)
- **Admin-Oberfläche**: Verwaltung von Inhalten (z.B. Artikel, Stationen, Zeitrahmen) 

Technisch ist die Idee: ein schlankes Rust/Axum Backend (REST-API) und ein separates Lit-Frontend.

## Installation

### Voraussetzungen

- Rust/Axum (für Admin/REST-API)
- PostgreSQL 
- Node.js + `pnpm` (für das Frontend, welches sich in einem anderen Repo befindet)


### Datenbank vorbereiten (PostgreSQL)

Für den lokalen Entwicklungsbetrieb verwenden wir PostgreSQL. Setze dafür `DATABASE_URL` und lege die DB + Tabellen an:

- `db-seeds/00-drop-tables.sql` leert das Schema.
- `db-seeds/01-tables.sql` baut das aktuelle Schema neu auf.
- `db-seeds/02-seed.sql` enthält Seed-Daten.

Die Testumgebung verwendet dieselben SQL-Dateien über `src/common/db_migrations.rs`, damit es keine zweite, abweichende Schemadefinition mehr gibt.
 codex resume 019d520f-69e0-7f20-b2fa-8ea86c7b2fb0