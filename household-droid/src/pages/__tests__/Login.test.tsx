import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import Login from "../Login.tsx";
import {
  createMemoryRouter,
  RouterProvider,
  createRoutesStub,
} from "react-router";

const createRouter = () => {
  return createMemoryRouter(
    [
      {
        path: "/login",
        Component: Login,
      },
    ],
    { initialEntries: ["/login"] },
  );
};

describe("Login", () => {
  it("renders", async () => {
    const screen = render(<RouterProvider router={createRouter()} />);
    await expect.element(screen.getByText(/Log In/i)).toBeInTheDocument();
  });
  it("render error on error", async () => {
    const Stub = createRoutesStub([
      {
        path: "/login",
        Component: Login,
        action: () => {
          return { error: { message: "big error" } };
        },
      },
    ]);
    const screen = render(<Stub initialEntries={["/login"]} />);
    const button = screen.getByRole("button", { name: "Log In" });
    await button.click();
    await expect.element(screen.getByText("big error")).toBeInTheDocument();
  });
});
