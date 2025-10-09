import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import { createRoutesStub } from "react-router";
import Metrics from "../Metrics.tsx";

describe("Metrics", () => {
  it("renders", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        Component: Metrics,

        loader: () => ({
          queryLatency: [
            {
              index: 0,
              range: "hi",
              count: 1,
            },
          ],
          ingestionLatency: [
            {
              index: 0,
              range: "2-3",
              count: 3,
            },
          ],
          queryTools: [
            {
              cnt_spns_with_tools: 2,
              cnt_spns_without_tools: 3,
              date: new Date(),
            },
          ],
        }),
        action: () => {},
      },
    ]);
    const screen = render(<Stub />);
    //rendered inside Table
    await expect
      .element(screen.getByText(/Without tool invocations/i))
      .toBeInTheDocument();
  });
});
