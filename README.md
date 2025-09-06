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

Docker images are built and available at `ghcr.io/danielhstahl/householdhelper:${tag}`.  The following environmental variables need to be defined:
* LM_STUDIO_ENDPOINT (defaults to "http://localhost:1234")
* OLLAMA_ENDPOINT (defaults to "http://localhost:11434")
* VECTOR_DATABASE_URL (defaults to "postgresql://postgres:yourpassword@localhost:5432")
* USER_DATABASE_URL (defaults to "sqlite://", in production this can be the same as VECTOR_DATABASE_URL)
* INIT_ADMIN_PASSWORD (required to start the app)
* HOST_STATIC.  Required to be set for the docker container to serve the compiled HTML.
* MLFLOW_TRACKING_URL.  Set for enabling traces
