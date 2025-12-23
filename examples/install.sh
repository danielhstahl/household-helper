## example of running this script: LLM_IP=[] PSQL_USER=stahlserver DOMAIN=draid.home RELEASE_TAG=v1.2.0 GENERATE_SSL=true ./install.sh
# Need to have a DNS resolver to point to draid.home
# Assumes debian OS
#

sudo apt update
sudo apt install uuid -y
sudo apt install nginx -y

base_url="https://github.com/danielhstahl/household-helper/releases/download/${RELEASE_TAG}"
ui_tar_name="draid.tar.gz"
url="${base_url}/${ui_tar_name}"
echo "downloading from ${url}"
curl -L -O $url

mkdir -p $HOME/draid
mkdir -p $HOME/draid/ssl
mkdir -p $HOME/draid/nginx
mkdir -p $HOME/draid/psqlstorage
sudo mkdir -p /usr/bin/draid

tar -xzvf ${ui_tar_name} # extracts in current dir
sudo mv dist /usr/bin/draid/
rm ${ui_tar_name}

sed -i -e "s/HOSTNAME/${DOMAIN}/g" docker-compose.yml
init_admin_password=$(uuid)
sed -i -e "s/[yourpassword]/${init_admin_password}/g" docker-compose.yml
jwt_secret=$(uuid)
sed -i -e "s/[yourjwtsecret]/${jwt_secret}/g" docker-compose.yml
psql_password=$(uuid)
sed -i -e "s/[yourpsqlpassword]/${psql_password}/g" docker-compose.yml
sed -i -e "s/[yourpsqluser]/${PSQL_USER}/g" docker-compose.yml
sed -i -e "s/[yourllmip]/${LLM_IP}/g" docker-compose.yml
sed -i -e "s@HOME@${HOME}@g" docker-compose.yml

sed -i -e "s/HOSTNAME/${DOMAIN}/g" nginx.app.conf
sed -i -e "s@HOME@${HOME}@g" nginx.app.conf
sed -i -e "s@HOME@${HOME}@g" nginx.service

GENERATE_SSL="${GENERATE_SSL:-false}"
if [ ${GENERATE_SSL} == "true" ]; then
    openssl req -x509 -newkey rsa:4096 -sha256 -days 3650 -nodes -keyout device.key -out device.crt -subj "/CN=$DOMAIN" -addext "subjectAltName=DNS:*.$DOMAIN"
    sudo mv device.key $HOME/draid/ssl/
    sudo mv device.crt $HOME/draid/ssl/
fi

sudo mv nginx.service /lib/systemd/system/draid-nginx.service
mv nginx.app.conf $HOME/draid/nginx/nginx.conf
mv init.sql $HOME/draid

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
