import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import AppBar from "../AppBar.tsx";
import { createRoutesStub } from "react-router";
describe("AppBar", () => {
  it("renders", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: () => {
          return (
            <AppBar
              threshold={500}
              isAdmin={true}
              agent="helper"
              sessionId="session123"
            />
          );
        },
      },
    ]);
    const screen = render(<Stub />);
    await expect.element(screen.getByText(/Draid/i)).toBeInTheDocument();
  });

  it("has correct settings when admin", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: () => {
          return (
            <AppBar
              threshold={500}
              isAdmin={true}
              agent="helper"
              sessionId="session123"
            />
          );
        },
      },
    ]);
    const screen = render(<Stub />);
    await expect
      .element(screen.getByRole("link", { name: "settings" }))
      .toBeInTheDocument();
  });
  it("has no settings when not admin", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: () => {
          return (
            <AppBar
              threshold={500}
              isAdmin={false}
              agent="helper"
              sessionId="session123"
            />
          );
        },
      },
    ]);
    const screen = render(<Stub />);
    await expect
      .element(screen.getByRole("link", { name: "settings" }))
      .not.toBeInTheDocument();
  });
});
