import { sendQuery, sendTutor } from "../services/Api";

const HELPER_INDEX = 0;
const TUTOR_INDEX = 1;
export const AgentSelectionsEnum = {
  HELPER_INDEX,
  TUTOR_INDEX,
} as const;

export type AgentSelections =
  (typeof AgentSelectionsEnum)[keyof typeof AgentSelectionsEnum];

export const invokeAgent = (selectedAgent: AgentSelections, query: string) => {
  switch (selectedAgent) {
    case AgentSelectionsEnum.HELPER_INDEX:
      return sendQuery(query);
    case AgentSelectionsEnum.TUTOR_INDEX:
      return sendTutor(query);
  }
};

export const getAgentName = (selectedAgent: AgentSelections) => {
  switch (selectedAgent) {
    case AgentSelectionsEnum.HELPER_INDEX:
      return "Helper";
    case AgentSelectionsEnum.TUTOR_INDEX:
      return "Tutor";
  }
};
