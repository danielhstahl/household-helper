import { describe, it, expect, vi, beforeEach } from "vitest";
import { getSessions, createSession, deleteSession } from "../api";
import { setupWorker } from "msw/browser";
import { http, HttpResponse } from "msw";

describe("api", () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it("getSessions returns sessions on success", async () => {
        const server = setupWorker(
            http.get("/api/session", () => {
                return HttpResponse.json([{ id: "session1" }]);
            })
        );
        await server.start({ quiet: true });
        const result = await getSessions("jwt");
        expect(result).toEqual([{ id: "session1" }]);
        server.stop();
    });

    it("getSessions throws error on failure", async () => {
        const server = setupWorker(
            http.get("/api/session", () => {
                return new HttpResponse("Error", { status: 500 });
            })
        );
        await server.start({ quiet: true });
        await expect(getSessions("jwt")).rejects.toThrow("Error");
        server.stop();
    });

    it("createSession returns session on success", async () => {
        const server = setupWorker(
            http.post("/api/session", () => {
                return HttpResponse.json({ id: "session1" });
            })
        );
        await server.start({ quiet: true });
        const result = await createSession("jwt");
        expect(result).toEqual({ id: "session1" });
        server.stop();
    });

    it("deleteSession returns status on success", async () => {
        const server = setupWorker(
            http.delete("/api/session/:id", () => {
                return HttpResponse.json({ status: "success" });
            })
        );
        await server.start({ quiet: true });
        const result = await deleteSession("session1", "jwt");
        expect(result).toEqual({ status: "success" });
        server.stop();
    });
});
