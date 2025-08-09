## install psql

### Linux
sudo apt update
echo | sudo apt install -y postgresql-common
echo | sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh
echo | sudo apt install postgresql-15-pgvector
sudo service postgresql start
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'yourpassword';"
sudo -u postgres psql -c "CREATE DATABASE vector_db;"

### Mac
brew install postgresql@17
brew link --overwrite postgresql@17
brew install pgvector
brew services start postgresql@17
psql -d postgres -c "ALTER USER postgres PASSWORD 'yourpassword';"
psql -d postgres -c "CREATE DATABASE vector_db;"
