import { AgentSelectionsEnum } from "../state/selectAgent";

export const getRedirectRoute = (
  agent: string | undefined,
  sessionId: string,
) => {
  //default to session that was most recently started
  return `/${agent || AgentSelectionsEnum.HELPER}/${sessionId}`;
};
