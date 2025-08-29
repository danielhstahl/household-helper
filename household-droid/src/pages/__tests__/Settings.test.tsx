import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import { createRoutesStub } from "react-router";
import Settings from "../Settings.tsx";

describe("MainChat", () => {
  it("renders", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: Settings,
        loader: () => [
          {
            id: 2,
            username: "hello",
            roles: ["admin"],
          },
        ],
        action: () => {},
      },
    ]);
    const screen = render(<Stub />);
    //rendered inside Table
    await expect.element(screen.getByText(/Username/i)).toBeInTheDocument();
  });
});
