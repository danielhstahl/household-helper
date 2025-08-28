import { getRedirectRoute } from "../routes";
import { describe, it, expect } from "vitest";

describe("getRedirectRoute", () => {
  it("returns full url if agent is defined", () => {
    expect(getRedirectRoute("helloworld", "sessionid1")).toEqual(
      "/helloworld/sessionid1",
    );
  });
  it("returns helper url if agent is not defined", () => {
    expect(getRedirectRoute(undefined, "sessionid1")).toEqual(
      "/helper/sessionid1",
    );
  });
});
