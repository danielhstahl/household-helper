import { describe, it, expect } from "vitest";
import { render } from "vitest-browser-react";
import Output, { MessageTypeEnum } from "../Output.tsx";

describe("Output", () => {
  it("renders", async () => {
    const messages = [
      {
        message_type: MessageTypeEnum.human,
        id: 1,
        content: "hello",
        timestamp: "sometime",
      },
    ];
    const screen = render(
      <Output
        messages={messages}
        isWaiting={false}
        latestText=""
        loading={false}
      />,
    );
    await expect.element(screen.getByText(/hello/i)).toBeInTheDocument();
  });
  it("shows loading if loading", async () => {
    const messages = [
      {
        message_type: MessageTypeEnum.human,
        id: 1,
        content: "hello",
        timestamp: "sometime",
      },
    ];
    const screen = render(
      <Output
        messages={messages}
        isWaiting={false}
        latestText=""
        loading={true}
      />,
    );
    await expect.element(screen.getByRole("progressbar")).toBeInTheDocument();
  });
  it("shows loading if isWaiting", async () => {
    const messages = [
      {
        message_type: MessageTypeEnum.human,
        id: 1,
        content: "hello",
        timestamp: "sometime",
      },
    ];
    const screen = render(
      <Output
        messages={messages}
        isWaiting={true}
        latestText=""
        loading={false}
      />,
    );
    await expect.element(screen.getByRole("progressbar")).toBeInTheDocument();
  });
  it("shows latest text if latestText", async () => {
    const messages = [
      {
        message_type: MessageTypeEnum.human,
        id: 1,
        content: "hello",
        timestamp: "sometime",
      },
    ];
    const screen = render(
      <Output
        messages={messages}
        isWaiting={false}
        latestText="goodbye"
        loading={false}
      />,
    );
    await expect.element(screen.getByText("goodbye")).toBeInTheDocument();
  });
});
