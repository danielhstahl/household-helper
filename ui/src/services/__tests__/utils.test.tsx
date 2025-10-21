import { parseText } from "../utils.tsx";
import { describe, it, expect } from "vitest";

describe("parseText", () => {
  it("replaces javascript", () => {
    expect(parseText("lorme ipsum etc ```javascriptconst v=5```")).toEqual(
      "lorme ipsum etc \n```javascript\nconst v=5\n```",
    );
  });
});
