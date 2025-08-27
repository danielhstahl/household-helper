from fastapi.testclient import TestClient
import os

os.environ["INIT_ADMIN_PASSWORD"] = "supersecret"


def test_read_session_no_auth():
    from index import app

    client = TestClient(app)
    response = client.get("/session")
    assert response.status_code == 401
    # assert response.json() == {"msg": "Hello World"}


def test_read_session():
    from index import app

    client = TestClient(app)
    data = {"username": "admin", "password": os.getenv("INIT_ADMIN_PASSWORD", "")}
    response = client.post("/token", data=data)
    session = client.get(
        "/session",
        headers={"Authorization": "Bearer " + response.json()["access_token"]},
    )
    assert session.status_code == 200
