FROM ghcr.io/astral-sh/uv:python3.13-bookworm-slim AS builder
ENV UV_COMPILE_BYTECODE=1 UV_LINK_MODE=copy
WORKDIR /app
RUN --mount=type=cache,target=/root/.cache/uv \
    --mount=type=bind,source=app/uv.lock,target=uv.lock \
    --mount=type=bind,source=app/pyproject.toml,target=pyproject.toml \
    uv sync --frozen --no-install-project --no-dev

# copy contents of app into code
ADD app /app/
RUN --mount=type=cache,target=/root/.cache/uv \
    uv sync --frozen --no-dev

FROM python:3.13-rc-slim-bookworm

# Copy the application from the builder
COPY --from=builder --chown=app:app /app /app

# Place executables in the environment at the front of the path
ENV PATH="/app/.venv/bin:$PATH"
ENV HOST_STATIC="yes"
# Run the FastAPI application by default
CMD ["fastapi", "run", "/app/main.py", "--port", "80", "--host", "0.0.0.0"]
