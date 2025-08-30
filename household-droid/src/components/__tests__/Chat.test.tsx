import { describe, it, expect, vi } from "vitest";
import { render } from "vitest-browser-react";
import Chat from "../Chat.tsx";
import { AgentSelectionsEnum } from "../../state/selectAgent.tsx";
import { userEvent } from "@vitest/browser/context";
describe("Chat", () => {
  it("renders", async () => {
    const onSubmit = vi.fn();
    const screen = render(
      <Chat onSubmit={onSubmit} selectedAgent={AgentSelectionsEnum.HELPER} />,
    );
    await expect
      .element(screen.getByText(/Chat or instruct your Helper/i))
      .toBeInTheDocument();
  });
  it("creates text and submits", async () => {
    const onSubmit = vi.fn();
    const screen = render(
      <Chat onSubmit={onSubmit} selectedAgent={AgentSelectionsEnum.HELPER} />,
    );
    await userEvent.keyboard("hello world!");
    await expect
      .element(screen.getByRole("textbox"))
      .toHaveValue("hello world!");
    await userEvent.keyboard("{Enter}");
    expect(onSubmit.mock.calls.length).toBe(1);
    expect(onSubmit.mock.calls[0][0]).toEqual(AgentSelectionsEnum.HELPER);
    expect(onSubmit.mock.calls[0][1]).toEqual("hello world!");
    await expect
      .element(screen.getByText(/hello world!/i))
      .not.toBeInTheDocument();
  });
});
