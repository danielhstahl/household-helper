import { sendQuery, sendTutor } from "../services/api.tsx";

const HELPER = "helper";
const TUTOR = "tutor";
export const AgentSelectionsEnum = {
  HELPER,
  TUTOR,
} as const;

export type AgentSelections =
  (typeof AgentSelectionsEnum)[keyof typeof AgentSelectionsEnum];

// eslint-disable-next-line react-refresh/only-export-components
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

// eslint-disable-next-line react-refresh/only-export-components
export const getAgentName = (selectedAgent: AgentSelections) => {
  switch (selectedAgent) {
    case AgentSelectionsEnum.HELPER:
      return "Helper";
    case AgentSelectionsEnum.TUTOR:
      return "Tutor";
  }
};
