## Household Helper

This is a slimmed-down bot that is intended to be a more capable Alexa.  Currently it is a standard chatbot with different "voices"/"agents" from different system prompts.  It is intended to be used with Ollama for RAG embeddings and LMStudio for the foundation model.

The intent is to create a local/private "intelligence appliance".

## Technology

* Server using Rocket and async-openai
* PSQL for user/vector database
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
sudo -u postgres psql -c "CREATE DATABASE draid;"
```

#### Mac

```sh
brew install postgresql@17
brew link --overwrite postgresql@17
brew install pgvector
brew services start postgresql@17
psql -d postgres -c "ALTER USER postgres PASSWORD 'yourpassword';"
psql -d postgres -c "CREATE DATABASE draid;"
psql -d postgres
\c driad
"CREATE EXTENSION vector;"
```

#### Into Docker

```sh
sudo docker exec -it [imghash] bash
psql -U [username]
psql -c "CREATE DATABASE draid;"
```

### Recommended dev command

Change directory to [draid](./draid), and run

`INIT_ADMIN_PASSWORD=[yourinitpassword] USER_DATABASE_URL=postgresql://postgres:[yourpassword]@localhost:5432 JWT_SECRET=[yourjwtsecret] cargo run`


## Deploy

There are two Docker images, one for the UI (static files), and one for the API (draid).  The UI Docker includes an nginx config that needs to point to the address of the API Docker.  These images are built and available at `ghcr.io/danielhstahl/householdhelper-ui:${tag}`, `ghcr.io/danielhstahl/householdhelper-draid:${tag}`.  The following environmental variables need to be defined on the draid Docker:
* OPEN_AI_COMPATABLE_ENDPOINT (defaults to "http://localhost:11434")
* ROCKET_DATABASES (eg, '{draid={url="postgresql://[yourpsqluser]:[yourpsqlpassword]@psqldb:5432/draid"}}')
* ROCKET_PORT (eg 8000)
* ROCKET_ADDRESS (eg, 0.0.0.0)
* JWT_SECRET (a secret string for authentication)


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
