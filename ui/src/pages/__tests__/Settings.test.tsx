import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import { createRoutesStub, redirect } from "react-router";
import Settings from "../Settings.tsx";
import Users from "../../components/Users.tsx";
import { RoleTypeEnum } from "../../services/models.tsx";

describe("Settings", () => {
  it("renders", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: Settings,
        children: [
          {
            loader: () => redirect("users"),
            index: true,
          },
          {
            path: "users",
            loader: () => [
              {
                id: 2,
                username: "hello",
                roles: [RoleTypeEnum.admin],
              },
            ],
            action: () => {},
            Component: Users,
          },
        ],
      },
    ]);
    const screen = render(<Stub />);
    //rendered inside Table
    await expect.element(screen.getByText(/Username/i)).toBeInTheDocument();
  });
});
