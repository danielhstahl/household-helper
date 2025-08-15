import Grid from "@mui/material/Grid";
//import { useState } from "react";
import AgentSelection from "./Agent";
const HELPER_INDEX = 0;
const TUTOR_INDEX = 1;
export const AgentSelectionsEnum = {
  HELPER_INDEX,
  TUTOR_INDEX,
} as const;
export type AgentSelections =
  (typeof AgentSelectionsEnum)[keyof typeof AgentSelectionsEnum];
interface AgentSelectionProps {
  selectedAgent: AgentSelections;
  setSelectedAgent: (agent: AgentSelections) => void;
}
const AgentSelectionOptions = ({
  selectedAgent,
  setSelectedAgent,
}: AgentSelectionProps) => {
  // likely use context later
  // for now, pass in as props
  return (
    <Grid container spacing={2} style={{ paddingTop: 20 }}>
      <Grid size={{ xs: 6, md: 4 }}>
        <AgentSelection
          isDefault={selectedAgent == HELPER_INDEX}
          agentType="Helper"
          agentDescription="This is a general purpose friendly household helper. It remembers past
        conversations. Think of it as an R2D2: a steady personality that can
        help navigate the minutiae that come up during your day."
          setDefault={() => setSelectedAgent(HELPER_INDEX)}
        />
      </Grid>
      <Grid size={{ xs: 6, md: 4 }}>
        <AgentSelection
          isDefault={selectedAgent == TUTOR_INDEX}
          agentType="Tutor"
          agentDescription="This is specifically intended to offer helpful assistance and
        tutoring for grade-school homework.  Won't give the answers though!"
          setDefault={() => setSelectedAgent(TUTOR_INDEX)}
        />
      </Grid>
    </Grid>
  );
};

export default AgentSelectionOptions;
