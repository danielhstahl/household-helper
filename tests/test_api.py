from fastapi.testclient import TestClient
from index import app

client = TestClient(app)


def test_read_session():
    response = client.get("/session")
    assert response.status_code == 200
    assert response.json() == {"msg": "Hello World"}
