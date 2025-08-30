import {
  loadSession,
  loadSessionsAndMessages,
  loadUser,
  logoutLoader,
  loadUsers,
} from "../loaders.tsx";
import { setLoggedInJwt } from "../../state/localState.tsx";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { http, HttpResponse } from "msw";
import { setupWorker } from "msw/browser";
import { DialogEnum } from "../../components/Output.tsx";
describe("loadSession", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });
  it("redirects to login if no jwt", async () => {
    setLoggedInJwt(null);
    const server = setupWorker(
      http.get("/session/recent", () => {
        return HttpResponse.json({
          id: "session123",
        });
      }),
      http.post("/session", () => {
        return HttpResponse.json({
          id: "session123",
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await loadSession({
      request: new Request("/dummyurl", { method: "GET" }),
      params: { agent: "helper" },
      context: {},
    });
    expect(result.headers.get("Location")).toEqual("/login");
    server.stop();
  });
  it("loads session if most recent session exists", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.get("/session/recent", () => {
        return HttpResponse.json({
          id: "session123",
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await loadSession({
      request: new Request("/dummyurl", { method: "GET" }),
      params: { agent: "helper" },
      context: {},
    });
    expect(result.headers.get("Location")).toEqual("/helper/session123");
    server.stop();
  });
  it("creates and loads session if no session exists", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.get("/session/recent", () => {
        return HttpResponse.json(null);
      }),
      http.post("/session", () => {
        return HttpResponse.json({
          id: "session123",
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await loadSession({
      request: new Request("/dummyurl", { method: "GET" }),
      params: { agent: "helper" },
      context: {},
    });
    expect(result.headers.get("Location")).toEqual("/helper/session123");
    server.stop();
  });
});

describe("loadSessionsAndMessages", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });
  it("redirects to login if no jwt", async () => {
    setLoggedInJwt(null);
    const server = setupWorker(
      http.get("/session", () => {
        return HttpResponse.json([
          {
            id: "session123",
          },
        ]);
      }),
      http.get("/messages/:sessionId", () => {
        return HttpResponse.json({
          id: "session123",
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await loadSessionsAndMessages({
      request: new Request("/dummyurl", { method: "GET" }),
      params: { agent: "helper" },
      context: {},
    });
    expect(result instanceof Response).toBeTruthy();
    if (result instanceof Response) {
      expect(result.headers.get("Location")).toEqual("/login");
    }
    server.stop();
  });
  it("loads sessions and messages", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.get("/session", () => {
        return HttpResponse.json([
          {
            id: "session123",
          },
          {
            id: "session456",
          },
        ]);
      }),
      http.get("/messages/:sessionId", () => {
        return HttpResponse.json([
          {
            id: 1,
            content: "hello world",
            timestamp: "time2",
            persona: "it",
          },
          {
            id: 1,
            content: "hello world",
            timestamp: "time1",
            persona: "it",
          },
        ]);
      }),
    );
    await server.start({ quiet: true });
    const result = await loadSessionsAndMessages({
      request: new Request("/dummyurl", { method: "GET" }),
      params: { sessionId: "session123" },
      context: {},
    });
    expect(result).toEqual({
      sessions: [
        {
          id: "session123",
        },
        {
          id: "session456",
        },
      ],
      messages: [
        {
          id: 1,
          content: "hello world",
          timestamp: "time1",
          persona: DialogEnum.It,
        },
        {
          id: 1,
          content: "hello world",
          timestamp: "time2",
          persona: DialogEnum.It,
        },
      ],
    });
    server.stop();
  });
});

describe("loadUser", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });
  it("redirects to login if no jwt", async () => {
    setLoggedInJwt(null);
    const server = setupWorker(
      http.get("/users/me", () => {
        return HttpResponse.json({
          id: 1,
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await loadUser();
    expect(result.headers.get("Location")).toEqual("/login");
    server.stop();
  });
  it("returns user if jwt", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.get("/users/me", () => {
        return HttpResponse.json({
          id: 1,
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await loadUser();
    expect(result).toEqual({ id: 1 });
    server.stop();
  });
});

describe("loadUsers", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });
  it("redirects to login if no jwt", async () => {
    setLoggedInJwt(null);
    const server = setupWorker(
      http.get("/users", () => {
        return HttpResponse.json({
          id: 1,
        });
      }),
    );
    await server.start({ quiet: true });
    const result = await loadUsers();
    expect(result.headers.get("Location")).toEqual("/login");
    server.stop();
  });
  it("returns users if jwt", async () => {
    setLoggedInJwt("dummyjwt");
    const server = setupWorker(
      http.get("/users", () => {
        return HttpResponse.json([
          {
            id: 1,
          },
        ]);
      }),
    );
    await server.start({ quiet: true });
    const result = await loadUsers();
    expect(result).toEqual([{ id: 1 }]);
    server.stop();
  });
});

describe("logoutLoader", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.clearAllMocks();
  });
  it("redirects to login if no jwt", async () => {
    setLoggedInJwt(null);
    const result = logoutLoader();
    expect(result.headers.get("Location")).toEqual("/login");
  });
  it("redirects to login if jwt", async () => {
    setLoggedInJwt("dummyjwt");
    const result = logoutLoader();
    expect(result.headers.get("Location")).toEqual("/login");
  });
});
