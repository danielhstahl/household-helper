import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import { createRoutesStub } from "react-router";
import MainPage from "../MainPage.tsx";

describe("MainPage", () => {
  it("renders settings when admin", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: MainPage,
        children: [{ path: "/", Component: () => <p>hello world</p> }],
        loader: () => ({
          id: 2,
          username: "hello",
          roles: ["admin"],
        }),
        action: () => {},
      },
    ]);
    const screen = render(<Stub />);
    await expect.element(screen.getByLabelText("settings")).toBeInTheDocument();
    await expect.element(screen.getByText("hello world")).toBeInTheDocument();
  });
  it("does not render settings when not an admin", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: MainPage,
        children: [{ path: "/", Component: () => <p>hello world</p> }],
        loader: () => ({
          id: 2,
          username: "hello",
          roles: ["tutor"],
        }),
        action: () => {},
      },
    ]);
    const screen = render(<Stub />);
    await expect
      .element(screen.getByLabelText("settings"))
      .not.toBeInTheDocument();
  });
});
