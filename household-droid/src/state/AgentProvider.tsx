import {
  useState,
  useContext,
  createContext,
  type PropsWithChildren,
} from "react";
import { type AgentSelections, AgentSelectionsEnum } from "./selectAgent";
interface AgentContextInit {
  state: AgentSelections;
  dispatch: (v: React.SetStateAction<AgentSelections>) => void;
}
const AgentContext = createContext<AgentContextInit>({
  state: AgentSelectionsEnum.HELPER_INDEX,
  // eslint-disable-next-line
  dispatch: (_v: React.SetStateAction<AgentSelections>) => {},
});

export const AgentProvider = ({ children }: PropsWithChildren) => {
  const [state, dispatch] = useState<AgentSelections>(
    AgentSelectionsEnum.HELPER_INDEX,
  );
  return (
    <AgentContext.Provider value={{ state, dispatch }}>
      {children}
    </AgentContext.Provider>
  );
};

// eslint-disable-next-line
export const useAgentParams = () => {
  return useContext(AgentContext);
};
