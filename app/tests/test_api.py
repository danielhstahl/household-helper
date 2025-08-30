import os

# app.dependency_overrides[engine] = test_engine
os.environ["INIT_ADMIN_PASSWORD"] = "supersecret"


def test_read_session_no_auth(client):
    response = client.get("/session")
    assert response.status_code == 401
    # assert response.json() == {"msg": "Hello World"}


def test_read_session(client):
    data = {"username": "admin", "password": os.getenv("INIT_ADMIN_PASSWORD", "")}
    response = client.post("/token", data=data)
    response_data = response.json()
    session = client.get(
        "/session",
        headers={"Authorization": "Bearer " + response_data["access_token"]},
    )
    assert session.status_code == 200
    assert session.json() == []


def test_create_session(client):
    data = {"username": "admin", "password": os.getenv("INIT_ADMIN_PASSWORD", "")}
    response = client.post("/token", data=data)
    response_data = response.json()
    session = client.post(
        "/session",
        headers={"Authorization": "Bearer " + response_data["access_token"]},
    )
    assert session.status_code == 200
    assert "session_start" in session.json()


def test_create_then_delete_session(client):
    data = {"username": "admin", "password": os.getenv("INIT_ADMIN_PASSWORD", "")}
    response = client.post("/token", data=data)
    response_data = response.json()
    session = client.post(
        "/session",
        headers={"Authorization": "Bearer " + response_data["access_token"]},
    )
    assert session.status_code == 200
    result = client.delete(
        f"/session/{session.json()['id']}",
        headers={"Authorization": "Bearer " + response_data["access_token"]},
    )
    assert result.status_code == 200
    assert result.json()["status"] == "success"


def test_create_user(client):
    data = {"username": "admin", "password": os.getenv("INIT_ADMIN_PASSWORD", "")}
    response = client.post("/token", data=data)
    response_data = response.json()
    user = client.post(
        "/users",
        json={"username": "mytest", "password": "mypassword", "roles": ["tutor"]},
        headers={"Authorization": "Bearer " + response_data["access_token"]},
    )
    assert user.status_code == 200


def test_create_update_delete_user(client):
    data = {"username": "admin", "password": os.getenv("INIT_ADMIN_PASSWORD", "")}
    response = client.post("/token", data=data)
    response_data = response.json()
    user = client.post(
        "/users",
        json={"username": "mytest", "password": "mypassword", "roles": ["tutor"]},
        headers={"Authorization": "Bearer " + response_data["access_token"]},
    )
    assert user.status_code == 200
    user_dict = user.json()

    user = client.patch(
        f"/users/{user_dict['id']}",
        json={
            "username": "mytest",
            "password": "mypassword2",
            "roles": ["tutor", "helper"],
        },
        headers={"Authorization": "Bearer " + response_data["access_token"]},
    )
    assert user.status_code == 200

    data = {"username": "mytest", "password": "mypassword2"}
    response = client.post("/token", data=data)
    response_data = response.json()

    user = client.get(
        "/users/me",
        headers={"Authorization": "Bearer " + response_data["access_token"]},
    )
    assert user.status_code == 200
    user_dict = user.json()
    for element in user_dict["roles"]:
        assert element in ["tutor", "helper"]
    assert len(user_dict["roles"]) == 2

    ## log back in as admin
    data = {"username": "admin", "password": os.getenv("INIT_ADMIN_PASSWORD", "")}
    response = client.post("/token", data=data)
    response_data = response.json()

    user = client.delete(
        f"/users/{user_dict['id']}",
        headers={"Authorization": "Bearer " + response_data["access_token"]},
    )
    assert user.status_code == 200
