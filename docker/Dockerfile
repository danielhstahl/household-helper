FROM python:3.13
RUN adduser --system --no-create-home householdhelper
WORKDIR /code
# copy contents of app into code
ADD app /code/
RUN mkdir static
# copy contents of dist into static
ADD ./ui/dist /code/static/
RUN pip3 install uv
RUN uv sync --no-dev
# don't run as root
USER householdhelper
CMD ["fastapi", "run", "main.py", "--port", "80"]
