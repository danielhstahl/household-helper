## Household Helper

This is a slimmed-down bot that is intended to be a more capable Alexa.  Currently it is a standard chatbot with different "voices"/"agents" from different system prompts.  It is intended to be used with Ollama for embeddings and LMStudio for the foundation model.

The intent is to create a local/private "intelligence appliance".

## Technology

* Llama-index for llm/agent orchestration
* FastAPI for the API layer
* PSQL for vector database
* SQLite or PSQL for user management
* React with Material UI for the client

## Development

### install psql

#### Linux

```sh
sudo apt update
echo | sudo apt install -y postgresql-common

echo | sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh

echo | sudo apt install postgresql-15-pgvector
sudo service postgresql start
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'yourpassword';"
sudo -u postgres psql -c "CREATE DATABASE vector_db;"
sudo -u postgres psql -c "CREATE DATABASE fastapi_db;"
```

#### Mac

```sh
brew install postgresql@17
brew link --overwrite postgresql@17
brew install pgvector
brew services start postgresql@17
psql -d postgres -c "ALTER USER postgres PASSWORD 'yourpassword';"
psql -d postgres -c "CREATE DATABASE vector_db;"
psql -d postgres -c "CREATE DATABASE fastapi_db;"
```

### Recommended dev command

`INIT_ADMIN_PASSWORD=[yourinitpassword] USER_DATABASE_URL=postgresql://postgres:[yourpassword]@localhost:5432 fastapi dev main.py`


## Deploy

There are two Docker images, one for the UI (static files) and one for the API.  The UI Docker includes an nginx config that needs to point to the address of the App Docker.  These images are built and available at `ghcr.io/danielhstahl/householdhelper-ui:${tag}` and `ghcr.io/danielhstahl/householdhelper-app:${tag}`.  The following environmental variables need to be defined on the app Docker:
* LM_STUDIO_ENDPOINT (defaults to "http://localhost:1234")
* OLLAMA_ENDPOINT (defaults to "http://localhost:11434")
* VECTOR_DATABASE_URL (defaults to "postgresql://postgres:yourpassword@localhost:5432")
* USER_DATABASE_URL (defaults to "sqlite://", in production this can be the same as VECTOR_DATABASE_URL)
* INIT_ADMIN_PASSWORD (required to start the app)
* MLFLOW_TRACKING_URL.  Set for enabling traces

In the UI docker:
* BACKEND_SERVICE.  Needs to be [ip/dns]:[port] of your app docker.

The [docker-compose](./docker/docker-compose.yml) file shows an example of how to orchestrate the containers.

## Helpful commands

If you are creating a self-signed certificate for local hosting:

`export DOMAIN_NAME=[yourdomainname]`

`export LOCAL_IP=[yourlocalip]`

`openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes \
  -keyout $DOMAIN_NAME.key -out $DOMAIN_NAME.crt -subj "/CN=$DOMAIN_NAME.local" \
  -addext "subjectAltName=DNS:*.$DOMAIN_NAME.local,IP:$LOCAL_IP"`
