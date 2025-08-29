import { sessionAction, loginAction, userAction } from "../actions.tsx";
import { setLoggedInJwt } from "../../state/localState.tsx";
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
      http.post("/session", () => {
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
      http.post("/session", () => {
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
      http.delete("/session/:id", () => {
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
    console.log(result);
    expect(result).toEqual({ status: "success" });

    server.stop();
  });
});
