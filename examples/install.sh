## example of running this script: DOMAIN=draid.home  ./install.sh
# Need to have a DNS resolver to point to draid.home
# Assumes debian OS
sed -i -e "s/HOSTNAME/${DOMAIN}/g" nginx.app.conf
sed -i -e "s@HOME@${HOME}@g" nginx.app.conf
sed -i -e "s@HOME@${HOME}@g" nginx.service
sed -i -e "s@HOME@${HOME}@g" docker-compose.yml
openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes -keyout device.key -out device.crt -subj "/CN=$DOMAIN" -addext "subjectAltName=DNS:*.$DOMAIN"
mkdir -p $HOME/draid
mkdir -p $HOME/draid/ssl
mkdir -p $HOME/draid/nginx
mkdir -p $HOME/draid/psqlstorage
sudo mv device.key $HOME/draid/ssl/
sudo mv device.crt $HOME/draid/ssl/
sudo mv nginx.service /lib/systemd/system/draid-nginx.service
sudo mv nginx.app.conf $HOME/draid/nginx/nginx.conf
sudo apt-get update
sudo apt-get install nginx -y
sudo systemctl daemon-reload
sudo systemctl enable draid-nginx
sudo systemctl restart draid-nginx


curl -sSL https://get.docker.com | sh
# requires restart to take effect
sudo usermod -aG docker ${USER}
sudo apt-get install docker-compose-plugin
sudo systemctl enable docker

# assumes a docker-compose.yml is in the same folder
docker compose up -d
