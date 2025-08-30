FROM python:3.13
RUN adduser --system --no-create-home householdhelper
WORKDIR /code
COPY uv.lock uv.lock
# copy contents into code
ADD app /code/
RUN mkdir static
ADD ./household-droid/dist /code/static/
RUN pip3 install uv
RUN uv sync --no-dev
# don't run as root
USER householdhelper
CMD ["fastapi", "run", "main.py", "--port", "80"]
