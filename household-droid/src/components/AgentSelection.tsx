import Grid from "@mui/material/Grid";
import AgentSelection from "./Agent";
import { AgentSelectionsEnum, getAgentName } from "../state/selectAgent";
import { useAgentParams } from "../state/AgentProvider";

const AgentSelectionOptions = ({ ref }: { ref: React.Ref<HTMLDivElement> }) => {
  const { state: selectedAgent, dispatch: setSelectedAgent } = useAgentParams();
  return (
    <Grid ref={ref} container spacing={2} style={{ paddingTop: 20 }}>
      <Grid
        size={{ sm: 6, md: 4 }}
        sx={{ display: { xs: "none", sm: "block" } }}
      >
        <AgentSelection
          isDefault={selectedAgent == AgentSelectionsEnum.HELPER_INDEX}
          agentType={getAgentName(AgentSelectionsEnum.HELPER_INDEX)}
          agentDescription="This is a general purpose friendly household helper. It remembers past
        conversations. Think of it as an R2D2: a steady personality that can
        help navigate the minutiae that come up during your day."
          setDefault={() => setSelectedAgent(AgentSelectionsEnum.HELPER_INDEX)}
        />
      </Grid>
      <Grid
        size={{ xs: 6, md: 4 }}
        sx={{ display: { xs: "none", sm: "block" } }}
      >
        <AgentSelection
          isDefault={selectedAgent == AgentSelectionsEnum.TUTOR_INDEX}
          agentType={getAgentName(AgentSelectionsEnum.TUTOR_INDEX)}
          agentDescription="This is specifically intended to offer helpful assistance and
        tutoring for grade-school homework.  Won't give the answers though!"
          setDefault={() => setSelectedAgent(AgentSelectionsEnum.TUTOR_INDEX)}
        />
      </Grid>
    </Grid>
  );
};

export default AgentSelectionOptions;
