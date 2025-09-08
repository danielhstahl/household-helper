FROM nginx:stable-alpine3.21-perl
# copy contents of dist into /usr/share/nginx/html
ADD ui/dist /usr/share/nginx/html
ADD docker/ui.nginx.conf /etc/nginx/conf.d/default.conf
