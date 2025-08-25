import Grid from "@mui/material/Grid";
import AgentSelection from "./Agent";
import { AgentSelectionsEnum, getAgentName } from "../state/selectAgent";
import { type AgentSelections } from "../state/selectAgent";

interface AgentSelectionProps {
  ref: React.Ref<HTMLDivElement>;
  selectedAgent: AgentSelections;
  selectedSessionId: string;
}
const AgentSelectionOptions = ({
  ref,
  selectedAgent,
  selectedSessionId,
}: AgentSelectionProps) => {
  return (
    <Grid ref={ref} container spacing={2} style={{ paddingTop: 20 }}>
      <Grid
        size={{ sm: 6, md: 4 }}
        sx={{ display: { xs: "none", sm: "block" } }}
      >
        <AgentSelection
          isDefault={selectedAgent == AgentSelectionsEnum.HELPER}
          agentType={getAgentName(AgentSelectionsEnum.HELPER)}
          agentDescription="This is a general purpose friendly household helper. It remembers past
        conversations. Think of it as an R2D2: a steady personality that can
        help navigate the minutiae that come up during your day."
          //setDefault={() => setSelectedAgent(AgentSelectionsEnum.HELPER_INDEX)}
          agent={AgentSelectionsEnum.HELPER}
          sessionId={selectedSessionId}
        />
      </Grid>
      <Grid
        size={{ xs: 6, md: 4 }}
        sx={{ display: { xs: "none", sm: "block" } }}
      >
        <AgentSelection
          isDefault={selectedAgent == AgentSelectionsEnum.TUTOR}
          agentType={getAgentName(AgentSelectionsEnum.TUTOR)}
          agentDescription="This is specifically intended to offer helpful assistance and
        tutoring for grade-school homework.  Won't give the answers though!"
          //setDefault={() => setSelectedAgent(AgentSelectionsEnum.TUTOR_INDEX)}
          agent={AgentSelectionsEnum.TUTOR}
          sessionId={selectedSessionId}
        />
      </Grid>
    </Grid>
  );
};

export default AgentSelectionOptions;
