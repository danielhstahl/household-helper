import { getAgentName, AgentSelectionsEnum } from "../selectAgent";
import { describe, it, expect } from "vitest";

describe("getAgentName", () => {
  it("returns Helper if helper is selected", () => {
    expect(getAgentName(AgentSelectionsEnum.HELPER)).toEqual("Helper");
  });
  it("returns Tutor if tutor is selected", () => {
    expect(getAgentName(AgentSelectionsEnum.TUTOR)).toEqual("Tutor");
  });
});
