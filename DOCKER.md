# Deployment with Docker

This project is designed to run in a containerized environment using Docker. The deployment methods are described below.

## 🚀 Method 1: With Docker Compose (Recommended)

This is the simplest and most recommended method. `docker-compose` orchestrates the image construction, network creation, and the startup of the application and database services in the correct order.

**1. Start the entire stack:**
Run this command in the project root. It will start everything in the background (`-d`).

```bash
docker compose up --build -d
```

**2. View logs:**
To view the logs of the application or the database in real time.

```bash
# View application logs
docker compose logs -f app

# View database logs
docker compose logs -f db
```

**3. Stop the stack:**
This will stop and remove the containers and the created network.

```bash
docker compose down
```

---

## 🔧 Method 2: Manual Docker Commands

If you do not have `docker compose` installed, you can manage the application's lifecycle with these manual `docker` commands.

### 1. Build the Image

This step packages the application into a Docker image. You only need to do this once or every time you change the code.

```bash
docker build -t rust-ai-auditor .
```

### 2. Start the Services

We need to create a network and then start the database and the application.

**Step 2.1: Create a Network**
So that the containers can communicate with each other by name.

```bash
docker network create auditor-network
```

**Step 2.2: Start the Database**
Starts a PostgreSQL container on the network we created.

```bash
docker run -d --name rust-ai-auditor-db --network auditor-network -p 5433:5432 -e POSTGRES_DB=ai_auditor -e POSTGRES_USER=postgres -e POSTGRES_PASSWORD=password postgres:18-alpine
```
*Note: Port `5433` is used on the host to avoid conflicts with local PostgreSQL instances.*

**Step 2.3: Start the Application**
Starts your application container, connecting it to the database.

```bash
docker run -d --name rust-ai-auditor-app --network auditor-network -p 3000:3000 -e DATABASE_URL="postgres://postgres:password@rust-ai-auditor-db:5432/ai_auditor" rust-ai-auditor
```
*The database URL points to `rust-ai-auditor-db`, the name of the database container.*

### 3. Check the Status

**View active containers:**
```bash
docker ps
```

**View application logs:**
Use this to confirm that the application started correctly and to debug errors.

```bash
docker logs rust-ai-auditor-app
```
*Add `-f` to follow the logs in real time.*

### 4. Stop and Clean Up

**Step 4.1: Stop the containers**
```bash
docker stop rust-ai-auditor-app rust-ai-auditor-db
```

**Step 4.2: Remove the containers**
Necessary if you want to start them again with the same names.
```bash
docker rm rust-ai-auditor-app rust-ai-auditor-db
```

**Step 4.3: Remove the network**
```bash
docker network rm auditor-network
```