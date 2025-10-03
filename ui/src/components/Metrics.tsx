import { BarChart } from "@mui/x-charts/BarChart";
import { useLoaderData } from "react-router";
import Grid from "@mui/material/Grid";
import Typography from "@mui/material/Typography";
type SpanLength = {
  range: string;
  frequency: number;
};
type SpanTools = {
  cnt_spns_with_tools: number;
  cnt_spns_without_tools: number;
  date: Date;
};
type SpanMetrics = {
  spanLength: readonly SpanLength[];
  spanTools: readonly SpanTools[];
};
const centerStyle = { display: "flex", justifyContent: "center" };
const Metrics = () => {
  const { spanLength, spanTools } = useLoaderData() as SpanMetrics;
  return (
    <>
      <Grid size={{ xs: 12 }}>
        <Typography style={centerStyle}>Tool use</Typography>
        <BarChart
          dataset={spanTools}
          height={300}
          xAxis={[{ dataKey: "date" }]}
          series={[
            {
              dataKey: "cnt_spns_with_tools",
              label: "Tool invocations",
            },
            {
              dataKey: "cnt_spns_without_tools",
              label: "Without tool invocations",
            },
          ]}
        />
      </Grid>
      <Grid size={{ xs: 12 }}>
        <Typography style={centerStyle}>Response time</Typography>
        <BarChart
          dataset={spanLength}
          height={300}
          xAxis={[{ dataKey: "range" }]}
          series={[
            {
              dataKey: "frequency",
            },
          ]}
        />
      </Grid>
    </>
  );
};
export default Metrics;
