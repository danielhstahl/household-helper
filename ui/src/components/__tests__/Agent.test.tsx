import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import AgentSelection from "../Agent.tsx";
import { AgentSelectionsEnum } from "../../state/selectAgent.tsx";
import { createRoutesStub } from "react-router";

describe("AgentSelectionOptions", () => {
  it("renders", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: () => {
          return (
            <AgentSelection
              agentType="Helper"
              agentDescription="hello world!"
              isDefault={true}
              agent={AgentSelectionsEnum.HELPER}
              sessionId="session123"
            />
          );
        },
      },
    ]);
    const screen = render(<Stub />);
    await expect.element(screen.getByText(/hello world!/i)).toBeInTheDocument();
  });
  it("has correct url", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: () => {
          return (
            <AgentSelection
              agentType="Helper"
              agentDescription="hello world!"
              isDefault={true}
              agent={AgentSelectionsEnum.HELPER}
              sessionId="session123"
            />
          );
        },
      },
    ]);
    const screen = render(<Stub />);
    await expect
      .element(screen.getByRole("link"))
      .toHaveAttribute("href", "/helper/session123");
  });
  it("is active if is default", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: () => {
          return (
            <AgentSelection
              agentType="Helper"
              agentDescription="hello world!"
              isDefault={true}
              agent={AgentSelectionsEnum.HELPER}
              sessionId="session123"
            />
          );
        },
      },
    ]);
    const screen = render(<Stub />);
    await expect
      .element(screen.getByRole("link"))
      .toHaveAttribute("data-active", "");
  });
  it("is not active if not default", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: () => {
          return (
            <AgentSelection
              agentType="Helper"
              agentDescription="hello world!"
              isDefault={false}
              agent={AgentSelectionsEnum.HELPER}
              sessionId="session123"
            />
          );
        },
      },
    ]);
    const screen = render(<Stub />);
    await expect
      .element(screen.getByRole("link"))
      .not.toHaveAttribute("data-active");
  });
});
