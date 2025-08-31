import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import AgentSelectionOptions from "../AgentSelection.tsx";
import { AgentSelectionsEnum } from "../../state/selectAgent.tsx";
import { useRef } from "react";
import { createRoutesStub } from "react-router";

describe("AgentSelectionOptions", () => {
  it("renders", async () => {
    //
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: () => {
          const ref = useRef(null);
          return (
            <AgentSelectionOptions
              ref={ref}
              selectedAgent={AgentSelectionsEnum.HELPER}
              selectedSessionId="session123"
            />
          );
        },
      },
    ]);
    const screen = render(<Stub />);
    await expect
      .element(screen.getByText(/Think of it as an R2D2/i))
      .toBeInTheDocument();
  });
});
