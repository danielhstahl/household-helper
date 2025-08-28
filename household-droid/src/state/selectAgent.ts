import { sendQuery, sendTutor } from "../services/api";

const HELPER = "helper";
const TUTOR = "tutor";
export const AgentSelectionsEnum = {
  HELPER,
  TUTOR,
} as const;

export type AgentSelections =
  (typeof AgentSelectionsEnum)[keyof typeof AgentSelectionsEnum];

export const invokeAgent = (
  selectedAgent: AgentSelections,
  query: string,
  jwt: string,
  sessionId: string,
) => {
  switch (selectedAgent) {
    case AgentSelectionsEnum.HELPER:
      return sendQuery(query, jwt, sessionId).then((r) => r.body!.getReader());
    case AgentSelectionsEnum.TUTOR:
      return sendTutor(query, jwt, sessionId).then((r) => r.body!.getReader());
  }
};

export const getAgentName = (selectedAgent: AgentSelections) => {
  switch (selectedAgent) {
    case AgentSelectionsEnum.HELPER:
      return "Helper";
    case AgentSelectionsEnum.TUTOR:
      return "Tutor";
  }
};
