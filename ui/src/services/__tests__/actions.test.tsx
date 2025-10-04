import { sessionAction, loginAction, userAction } from "../actions.tsx";
import { getLoggedInJwt, setLoggedInJwt } from "../../state/localState.tsx";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { setupWorker } from "msw/browser";

describe("sessionAction", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });
  it("redirects to login if no jwt", async () => {
    setLoggedInJwt(null);
    const server = setupWorker(
      http.post("/api/session", () => {
        return HttpResponse.json({
          id: "session123",
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await sessionAction({
      request: new Request("/dummyurl", { method: "POST" }),
      params: { agent: "helper" },
      context: {},
    });

    expect(result.headers.get("Location")).toEqual("/login");

    server.stop();
  });
  it("returns new session if POST and JWT", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.post("/api/session", () => {
        return HttpResponse.json({
          id: "session123",
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await sessionAction({
      request: new Request("/dummyurl", { method: "POST" }),
      params: { agent: "helper" },
      context: {},
    });

    expect(result.headers.get("Location")).toEqual("/helper/session123");

    server.stop();
  });
  it("returns deletes session if DELETE and JWT", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.delete("/api/session/:id", () => {
        return HttpResponse.json({
          status: "success",
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await sessionAction({
      request: new Request("/dummyurl", { method: "DELETE" }),
      params: { agent: "helper" },
      context: {},
    });
    expect(result).toEqual({ status: "success" });
    server.stop();
  });
});

describe("loginAction", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });
  it("returns jwt on successful login", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.post("/api/login", () => {
        return HttpResponse.json({
          access_token: "helloworld",
        });
      }),
    );
    await server.start({ quiet: true });
    const formData = new FormData();
    formData.append("username", "hello");
    formData.append("password", "world");
    const result = await loginAction({
      request: new Request("/login", { method: "POST", body: formData }),
      params: {},
      context: {},
    });
    expect(getLoggedInJwt()).toEqual("helloworld");
    expect(result instanceof Response).toBeTruthy();
    if (result instanceof Response) {
      expect(result.headers.get("Location")).toEqual("/");
    }
    server.stop();
  });
  it("returns error if not authenticated", async () => {
    const server = setupWorker(
      http.post("/api/login", () => {
        return HttpResponse.text("somehtml", { status: 401 });
      }),
    );
    await server.start({ quiet: true });
    const formData = new FormData();
    formData.append("username", "hello");
    formData.append("password", "world");
    const result = await loginAction({
      request: new Request("/login", { method: "POST", body: formData }),
      params: {},
      context: {},
    });
    expect(getLoggedInJwt()).toEqual(null);
    if (result instanceof Response) {
      console.log(result.status);
      console.log(result.body);
    }
    expect(result).toEqual({ error: Error("Unauthorized") });

    server.stop();
  });
});

describe("userAction", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });
  it("redirects to login if no jwt", async () => {
    setLoggedInJwt(null);
    const server = setupWorker(
      http.post("/api/users", () => {
        return HttpResponse.json({
          id: "session123",
        });
      }),
    );
    await server.start({ quiet: true });
    const formData = new FormData();
    formData.append(
      "data",
      JSON.stringify({
        username: "hello",
        password: "world",
        roles: ["admin"],
      }),
    );
    const result = await userAction({
      request: new Request("/dummyurl", { method: "POST", body: formData }),
      params: { agent: "helper" },
      context: {},
    });

    expect(result.headers.get("Location")).toEqual("/login");

    server.stop();
  });
  it("returns new user if POST and JWT", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.post("/api/user", () => {
        return HttpResponse.json({
          id: 2,
          username: "hello",
          roles: ["admin"],
        });
      }),
    );
    await server.start({ quiet: true });
    const formData = new FormData();
    formData.append(
      "data",
      JSON.stringify({
        username: "hello",
        password: "world",
        roles: ["admin"],
      }),
    );
    const result = await userAction({
      request: new Request("/dummyurl", { method: "POST", body: formData }),
      params: {},
      context: {},
    });
    expect(result).toEqual({
      id: 2,
      username: "hello",
      roles: ["admin"],
    });

    server.stop();
  });
  it("updates new user if PATCH and JWT", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.patch("/api/user/:id", ({ params }) => {
        const { id } = params;
        return HttpResponse.json({
          id: parseInt(id as string),
          username: "hello",
          roles: ["admin"],
        });
      }),
    );
    await server.start({ quiet: true });
    const formData = new FormData();
    formData.append(
      "data",
      JSON.stringify({
        id: 2,
        username: "hello",
        password: "world",
        roles: ["admin"],
      }),
    );
    const result = await userAction({
      request: new Request("/dummyurl", { method: "PATCH", body: formData }),
      params: {},
      context: {},
    });
    expect(result).toEqual({
      id: 2,
      username: "hello",
      roles: ["admin"],
    });
    server.stop();
  });

  it("deletes user if DELETE and JWT", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.delete("/api/user/:id", () => {
        return HttpResponse.json({
          status: "success",
        });
      }),
    );
    await server.start({ quiet: true });
    const formData = new FormData();
    formData.append(
      "data",
      JSON.stringify({
        id: 2,
      }),
    );
    const result = await userAction({
      request: new Request("/dummyurl", { method: "DELETE", body: formData }),
      params: {},
      context: {},
    });
    expect(result).toEqual({ status: "success" });
    server.stop();
  });
});
