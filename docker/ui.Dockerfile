FROM nginx:stable-alpine3.21-perl
# copy contents of dist into /usr/share/nginx/html
ADD ui/dist /usr/share/nginx/html
ADD docker/ui-docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
ADD docker/ui.nginx.conf.template /etc/nginx/nginx.conf.template

RUN mkdir -p /var/run/nginx && chown -R nginx:nginx /var/run/nginx
RUN chown -R nginx:nginx /var/cache/nginx && \
    chown -R nginx:nginx /var/log/nginx && \
    chown -R nginx:nginx /etc/nginx/ && \
    chmod +x /usr/local/bin/docker-entrypoint.sh

# Switch to the non-root user
USER nginx

# Final entrypoint and command
ENTRYPOINT ["/usr/local/bin/docker-entrypoint.sh"]
CMD ["nginx", "-g", "daemon off;"]
