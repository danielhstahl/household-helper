import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import App from "../App.tsx";
import MainPage from "../pages/MainPage.tsx";
import { createMemoryRouter, RouterProvider } from "react-router";
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
    await expect.element(screen.getByText(/Draid/i)).toBeInTheDocument();
  });
});
