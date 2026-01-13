# rust-ai-auditor

## Database Setup

### 1. Install PostgreSQL with Docker

Create and run the PostgreSQL container:

```bash
docker run --name rust-db \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=ai_auditor \
  -p 5432:5432 \
  -d postgres
```

### 2. Verify Installation

Check that the container is running:

```bash
docker ps
```

You should see `rust-db` in the list.

### 3. Configure Environment

Create the `.env` file in the project root:

```bash
touch .env
```

Add the database connection string to `.env`:

```
DATABASE_URL=postgres://postgres:password@localhost:5432/ai_auditor
```

### 4. Connect to Database

To access the PostgreSQL interactive terminal:

```bash
docker exec -it rust-db psql -U postgres -d ai_auditor
```

### 5. Useful PostgreSQL Commands

Once connected to `psql`:

```sql
-- List all databases
\l

-- List all tables
\dt

-- Describe a table structure
\d table_name

-- List all schemas
\dn

-- Show current database
SELECT current_database();

-- Show all users
\du

-- Execute SQL file
\i path/to/file.sql

-- Quit psql
\q
```

### 6. Docker Commands

```bash
# Start the container
docker start rust-db

# Stop the container
docker stop rust-db

# Remove the container
docker rm rust-db

# View container logs
docker logs rust-db

# View container logs in real-time
docker logs -f rust-db
```

## API Usage

### Create Audit

Create a new AI code audit record.

**Endpoint:** `POST /audit`

**Request:**

```bash
curl -X POST http://localhost:3000/audit -H "Content-Type: application/json" -d '{"prompt":"Crea una función que sume dos números","codigo_generado":"fn suma(a: i32, b: i32) -> i32 { a + b }","es_valido":true,"error_compilacion":null}'
```

**Response (201 Created):**

```json
{
  "id": "bfc949cc-743c-44d5-bc94-8ada8fed8fbc",
  "prompt": "Crea una función que sume dos números",
  "codigo_generado": "fn suma(a: i32, b: i32) -> i32 { a + b }",
  "es_valido": true,
  "error_compilacion": null,
  "created_at": "2026-01-13T20:02:18.213246Z"
}
```

### Verify Data in Database

To check the stored audits:

```bash
docker exec -it rust-db psql -U postgres -d ai_auditor -c "SELECT id, prompt, es_valido, created_at FROM ai_audits;"
```