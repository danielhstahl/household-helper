import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import { createRoutesStub } from "react-router";
import Users from "../Users.tsx";
import { RoleTypeEnum } from "../../services/models.tsx";

describe("MainChat", () => {
  it("renders", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: Users,

        loader: () => [
          {
            id: 2,
            username: "hello",
            roles: [RoleTypeEnum.admin],
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
