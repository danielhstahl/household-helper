import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import { createRoutesStub } from "react-router";
import KnowledgeBase from "../KnowledgeBase.tsx";

describe("KnowledgeBase", () => {
  it("renders", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        loader: () => Promise.resolve([{ id: 1, name: "recipe" }]),
        Component: KnowledgeBase,
      },
    ]);

    const screen = render(<Stub initialEntries={["/"]} />);
    await expect
      .element(screen.getByText(/Knowledge base recipe/i))
      .toBeInTheDocument();
  });
  it("renders multiple kbs", async () => {
    const Stub = createRoutesStub([
      {
        path: "/",
        loader: () =>
          Promise.resolve([
            { id: 1, name: "recipe" },
            { id: 1, name: "gardening" },
          ]),
        Component: KnowledgeBase,
      },
    ]);

    const screen = render(<Stub initialEntries={["/"]} />);
    await expect
      .element(screen.getByText(/Knowledge base recipe/i))
      .toBeInTheDocument();
    await expect
      .element(screen.getByText(/Knowledge base gardening/i))
      .toBeInTheDocument();
  });
});
