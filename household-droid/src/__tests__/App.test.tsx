import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import App from "../App.tsx";
import { createMemoryRouter, RouterProvider } from "react-router";
import MainPage from "../pages/MainPage.tsx";
const createRouter = () => {
  return createMemoryRouter([
    {
      path: "/",
      Component: App,
      children: [
        {
          path: "/",
          Component: MainPage,
          loader: async () => ({
            roles: ["admin"],
            id: 3,
            username: "admin",
          }),
          children: [
            {
              path: "/",
              Component: () => <p>hello</p>,
            },
          ],
        },
      ],
    },
  ]);
};

describe("App", () => {
  it("renders", async () => {
    const screen = render(<RouterProvider router={createRouter()} />);
    await expect
      .element(screen.getByText(/Household Droid/i))
      .toBeInTheDocument();
  });
});
