import { describe, it, expect, vi } from "vitest";
import { render } from "vitest-browser-react";
import { createRoutesStub } from "react-router";
import SessionSelection from "../SessionSelection.tsx";
describe("SessionSelection", () => {
  it("renders", async () => {
    const Stub = createRoutesStub([
      {
        path: "/helper/session123",
        Component: () => {
          const sessions = [
            { id: "session123", session_start: "2025-08-30T13:00:18.350Z" },
          ];
          return (
            <SessionSelection
              sessions={sessions}
              selectedSessionId="session123"
            />
          );
        },
        action: () => {},
      },
    ]);

    const screen = render(<Stub initialEntries={["/helper/session123"]} />);
    await expect.element(screen.getByText(/Sessions/i)).toBeInTheDocument();
    await expect
      .element(screen.getByText(/2025-08-30T13:00/i))
      .toBeInTheDocument();
  });

  it("doesnt allow deletion if selected", async () => {
    const action = vi.fn();
    const Stub = createRoutesStub([
      {
        path: "/helper/:sessionId",
        Component: ({ params }) => {
          const sessions = [
            { id: "session123", session_start: "2025-08-30T13:00:18.350Z" },
            { id: "session456", session_start: "2025-08-30T12:00:18.350Z" },
          ];
          return (
            <SessionSelection
              sessions={sessions}
              selectedSessionId={params.sessionId as string}
            />
          );
        },
        action,
      },
    ]);

    const screen = render(<Stub initialEntries={["/helper/session123"]} />);

    await expect
      .element(screen.getByRole("button", { name: "delete" }).first())
      .toHaveAttribute("disabled", "");
    await expect
      .element(screen.getByRole("button", { name: "delete" }).nth(1))
      .not.toHaveAttribute("disabled");
  });

  it("doesnt allow deletion if selected", async () => {
    const action = vi.fn();
    const Stub = createRoutesStub([
      {
        path: "/helper/:sessionId",
        Component: ({ params }) => {
          const sessions = [
            { id: "session123", session_start: "2025-08-30T13:00:18.350Z" },
            { id: "session456", session_start: "2025-08-30T12:00:18.350Z" },
          ];
          return (
            <SessionSelection
              sessions={sessions}
              selectedSessionId={params.sessionId as string}
            />
          );
        },
        action,
      },
    ]);

    const screen = render(<Stub initialEntries={["/helper/session123"]} />);
    const deleteButton = screen.getByRole("button", { name: "delete" }).nth(1);
    await deleteButton.click();
    expect(action.mock.calls.length).toEqual(1);
    expect(action.mock.calls[0][0].params.sessionId).toEqual("session456");
    expect(
      action.mock.calls[0][0].request.url.endsWith("/helper/session456"),
    ).toBeTruthy();
  });
  it("redirects on click of another session", async () => {
    const action = vi.fn();
    const Stub = createRoutesStub([
      {
        path: "/helper/:sessionId",
        Component: ({ params }) => {
          const sessions = [
            { id: "session123", session_start: "2025-08-30T13:00:18.350Z" },
            { id: "session456", session_start: "2025-08-30T12:00:18.350Z" },
          ];
          return (
            <SessionSelection
              sessions={sessions}
              selectedSessionId={params.sessionId as string}
            />
          );
        },
        action,
      },
    ]);

    const screen = render(<Stub initialEntries={["/helper/session123"]} />);
    const session456 = screen.getByRole("listitem").nth(1);
    await session456.click();

    await expect
      .element(screen.getByRole("button", { name: "delete" }).nth(1))
      .toHaveAttribute("disabled", "");
  });
});
