import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import { createRoutesStub, Outlet } from "react-router";
import { useRef } from "react";
import MainChat from "../MainChat.tsx";

describe("MainChat", () => {
  it("renders", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: () => (
          <Outlet context={{ agentSelectionRef: useRef(null) }} />
        ),
        children: [
          {
            path: "/:agent/:sessionId",
            Component: MainChat,
            loader: () => ({
              sessions: [],
              messages: [],
            }),
            action: () => {},
          },
        ],
      },
    ]);
    const screen = render(<Stub initialEntries={["/helper/session123"]} />);
    //rendered inside SessionSelection
    await expect.element(screen.getByText(/Sessions/i)).toBeInTheDocument();
  });
});
