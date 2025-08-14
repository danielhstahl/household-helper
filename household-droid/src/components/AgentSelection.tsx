import Grid from "@mui/material/Grid";
import { useState } from "react";
import AgentSelection from "./Agent";
const HELPER_INDEX = 0;
const TUTOR_INDEX = 1;
const AgentSelectionOptions = () => {
  //likely use context later
  // for now, set state

  const [selectedAgent, setSelectedAgent] = useState<number>(HELPER_INDEX);
  return (
    /*<Stack direction="row" spacing={2} style={{ paddingTop: 20 }}>
      <AgentSelection
        isDefault={selectedAgent == HELPER_INDEX}
        agentType="Helper"
        agentDescription="This is a general purpose friendly household helper. It remembers past
        conversations. Think of it as an R2D2: a steady personality that can
        help navigate the minutae that come up during your day."
        setDefault={(isChecked) => isChecked && setSelectedAgent(HELPER_INDEX)}
      />
      <AgentSelection
        isDefault={selectedAgent == TUTOR_INDEX}
        agentType="Tutor"
        agentDescription="This is specifically intended to offer helpful assistance and
        tutoring for grade-school homework.  Won't give the answers though!"
        setDefault={(isChecked) => isChecked && setSelectedAgent(TUTOR_INDEX)}
      />
    </Stack>*/
    <Grid container spacing={2} style={{ paddingTop: 20 }}>
      <Grid size={{ xs: 6, md: 4 }}>
        <AgentSelection
          isDefault={selectedAgent == HELPER_INDEX}
          agentType="Helper"
          agentDescription="This is a general purpose friendly household helper. It remembers past
        conversations. Think of it as an R2D2: a steady personality that can
        help navigate the minutae that come up during your day."
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
