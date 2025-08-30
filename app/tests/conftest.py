import pytest
from models import Base
from sqlalchemy import create_engine, StaticPool
from main import create_fastapi
from user import get_db
from fastapi.testclient import TestClient
from sqlalchemy.orm import sessionmaker


@pytest.fixture(name="engine")
def engine_fixture():
    engine = create_engine(
        "sqlite:///:memory:",
        connect_args={"check_same_thread": False},
        poolclass=StaticPool,
    )
    Base.metadata.create_all(bind=engine)
    return engine


@pytest.fixture(name="session")
def session_fixture(engine):
    # Create a session for each test
    SessionLocal = sessionmaker(autocommit=False, autoflush=False, bind=engine)
    session = SessionLocal()
    try:
        yield session
    finally:
        session.close()


@pytest.fixture(name="client")
def test_app(engine, session):
    def override_get_db():
        try:
            yield session
        finally:
            session.rollback()  # Rollback any uncommitted changes after test
            session.close()

    app = create_fastapi(engine)
    app.dependency_overrides[get_db] = override_get_db
    with TestClient(app) as client:
        yield client
    app.dependency_overrides.clear()
